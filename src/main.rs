use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::{PresentMode, PrimaryWindow, Window, WindowMode};

use bevy_asset_loader::prelude::*;

mod utils;

const TURRET_Z_OFFSET: Vec3 = Vec3::new(0.0, 0.0, 10.0);

#[derive(Component)]
pub struct MainCamera;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    Gaming,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "ship.png")]
    pub ship: Handle<Image>,

    #[asset(path = "turret.png")]
    pub turret: Handle<Image>,

    #[asset(path = "water.png")]
    pub water: Handle<Image>,
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Turret {
    pub offset: Vec3,
}

impl Default for Turret {
    fn default() -> Self {
        Self {
            offset: Vec3::default(),
        }
    }
}

#[derive(Component)]
pub struct ShipStats {
    delta_steering: f32,
    delta_speed: f32,
    current_speed: f32,

    min_speed: f32,
    max_speed: f32,
}

#[derive(Resource, Default)]
pub struct MouseWorldCoords(Vec2);

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            delta_steering: 4.0,
            delta_speed: 100.0,
            current_speed: 0.0,
            min_speed: -150.0,
            max_speed: 500.0,
        }
    }
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Gaming),
        )
        .add_collection_to_loading_state::<_, GameAssets>(GameState::AssetLoading)
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoVsync,
                        mode: WindowMode::Fullscreen,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .build(),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .init_resource::<MouseWorldCoords>()
        .add_systems(
            OnEnter(GameState::Gaming),
            (spawn_camera, spawn_player, spawn_turret, spawn_water_tiles),
        )
        .add_systems(
            Update,
            (
                steer_player,
                accelerate_player,
                fetch_scroll_events,
                move_ships,
                reposition_turrets,
                rotate_turrets,
                fetch_mouse_world_coords,
                move_camera,
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(750.0);
    commands.spawn((MainCamera, camera));
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Player {},
        ShipStats::default(),
        SpriteBundle {
            texture: assets.ship.clone(),
            ..default()
        },
    ));
}

fn spawn_turret(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        SpriteBundle {
            texture: assets.turret.clone(),
            ..default()
        },
        Turret::default(),
    ));
}

fn fetch_mouse_world_coords(
    mut mouse_coords: ResMut<MouseWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mouse_coords.0 = world_position;
    }
}

fn fetch_scroll_events(
    mut scroll_evr: EventReader<MouseWheel>,
    mut q_projection: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    for ev in scroll_evr.read() {
        let mut projection = q_projection.single_mut();
        match ev.unit {
            MouseScrollUnit::Line => {
                let scroll = if ev.y > 0.0 { -1.0 } else { 1.0 };
                projection.scale = (projection.scale + scroll).clamp(1.0, 10.0);
            }
            MouseScrollUnit::Pixel => {
                let scroll = if ev.y > 0.0 { -1.0 } else { 1.0 };
                projection.scale = (projection.scale + scroll).clamp(1.0, 10.0);
            }
        }
    }
}

fn move_camera(
    mut q_camera: Query<
        (&mut Transform, &OrthographicProjection),
        (With<MainCamera>, Without<Player>),
    >,
    q_player: Query<&Transform, With<Player>>,
    mouse_coords: Res<MouseWorldCoords>,
) {
    let (mut camera_transform, projection) = q_camera.single_mut();
    let player_pos = q_player.single().translation;

    // camera_transform.translation = player_pos;
    camera_transform.translation =
        player_pos + (mouse_coords.0.extend(0.0) - player_pos) / 4.0 / projection.scale;
}

fn steer_player(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(&mut Transform, &ShipStats)>,
) {
    let (mut transform, ship_stats) = player.single_mut();

    let mut steer_direction = 0.0;
    if keys.pressed(KeyCode::A) {
        steer_direction += 1.0;
    }
    if keys.pressed(KeyCode::D) {
        steer_direction -= 1.0;
    }

    if steer_direction == 0.0 {
        return;
    }

    let rotation = ship_stats.delta_steering * steer_direction * time.delta_seconds();
    transform.rotate_z(rotation);
}

fn accelerate_player(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut ShipStats>,
) {
    let mut ship_stats = player.single_mut();

    let mut acceleration = 0.0;
    if keys.pressed(KeyCode::W) {
        acceleration += 1.0;
    }
    if keys.pressed(KeyCode::S) {
        acceleration -= 1.0;
    }

    ship_stats.current_speed = (ship_stats.current_speed
        + acceleration * ship_stats.delta_speed * time.delta_seconds())
    .clamp(ship_stats.min_speed, ship_stats.max_speed);
}

fn move_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &ShipStats)>) {
    for (mut transform, ship_stats) in &mut ships {
        let direction = transform.local_y();
        transform.translation += direction * ship_stats.current_speed * time.delta_seconds();
    }
}

fn reposition_turrets(
    mut turrets: Query<(&mut Transform, &Turret), Without<Player>>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player_transform = q_player.single();
    for (mut turret_transform, turret) in &mut turrets {
        turret_transform.translation = player_transform.translation
            + player_transform.rotation.mul_vec3(turret.offset)
            + TURRET_Z_OFFSET;
    }
}

fn rotate_turrets(
    mut turrets: Query<&mut Transform, With<Turret>>,
    mouse_coords: Res<MouseWorldCoords>,
) {
    for mut turret in &mut turrets {
        turret.rotation =
            utils::quat_from_vec2(-1.0 * (mouse_coords.0 - turret.translation.truncate()).perp());
    }
}

fn spawn_water_tiles(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((SpriteBundle {
        texture: assets.water.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -100.0)),
        ..default()
    },));
}
