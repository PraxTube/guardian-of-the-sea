use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, Window, WindowMode};

use bevy_asset_loader::prelude::*;
use turret::{SpawnTurretsEvent, Turret};

mod projectile;
mod turret;
mod utils;
mod world;

use world::MainCamera;

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
    #[asset(path = "boat.png")]
    pub boat: Handle<Image>,

    #[asset(path = "turret.png")]
    pub turret: Handle<Image>,

    #[asset(path = "water.png")]
    pub water: Handle<Image>,

    #[asset(path = "rocket.png")]
    pub rocket: Handle<Image>,
}

#[derive(Component)]
pub struct Player {
    active_momentum: bool,
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
                        present_mode: PresentMode::Fifo,
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
        .add_plugins((
            projectile::ProjectilePlugin,
            turret::TurretPlugin,
            world::GuardianWorldPlugin,
        ))
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .init_resource::<MouseWorldCoords>()
        .add_systems(
            OnEnter(GameState::Gaming),
            (
                // spawn_player_small,
                spawn_player_big,
                spawn_water_tiles,
            ),
        )
        .add_systems(
            Update,
            (
                steer_player,
                accelerate_player,
                toggle_player_active_momentum,
                reduce_player_speed,
                fetch_scroll_events,
                fetch_mouse_world_coords,
                move_ships,
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .run();
}

fn spawn_player_small(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventWriter<SpawnTurretsEvent>,
) {
    commands.spawn((
        Player {
            active_momentum: true,
        },
        ShipStats::default(),
        SpriteBundle {
            texture: assets.ship.clone(),
            ..default()
        },
    ));
    ev_spawn_turrets.send(SpawnTurretsEvent {
        turrets: vec![Turret::default()],
    })
}

fn spawn_player_big(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventWriter<SpawnTurretsEvent>,
) {
    let mut ship_stats = ShipStats::default();
    ship_stats.delta_speed *= 0.75;
    ship_stats.delta_steering *= 0.5;
    commands.spawn((
        Player {
            active_momentum: true,
        },
        ship_stats,
        SpriteBundle {
            texture: assets.boat.clone(),
            ..default()
        },
    ));
    ev_spawn_turrets.send(SpawnTurretsEvent {
        turrets: vec![
            Turret::new(Vec2::new(16.0, 16.0)),
            Turret::new(Vec2::new(-16.0, 16.0)),
            Turret::new(Vec2::new(16.0, -16.0)),
            Turret::new(Vec2::new(-16.0, -16.0)),
            Turret::new(Vec2::new(-16.0, 48.0)),
            Turret::new(Vec2::new(16.0, 48.0)),
        ],
    })
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

fn toggle_player_active_momentum(keys: Res<Input<KeyCode>>, mut q_player: Query<&mut Player>) {
    if !keys.just_pressed(KeyCode::T) {
        return;
    }

    let mut player = q_player.single_mut();
    player.active_momentum = !player.active_momentum;
}

fn move_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &ShipStats)>) {
    for (mut transform, ship_stats) in &mut ships {
        let direction = transform.local_y();
        transform.translation += direction * ship_stats.current_speed * time.delta_seconds();
    }
}

fn reduce_player_speed(time: Res<Time>, mut q_player: Query<(&mut ShipStats, &Player)>) {
    let (mut ship_stats, player) = q_player.single_mut();
    if player.active_momentum || ship_stats.current_speed == 0.0 {
        return;
    }

    let reduction = ship_stats.delta_speed / 2.0 * time.delta_seconds();
    if ship_stats.current_speed > 0.0 {
        ship_stats.current_speed = (ship_stats.current_speed - reduction).max(0.0);
    } else {
        ship_stats.current_speed = (ship_stats.current_speed + reduction).min(0.0);
    }
}

fn spawn_water_tiles(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((SpriteBundle {
        texture: assets.water.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -100.0))
            .with_scale(Vec3::splat(10.0)),
        ..default()
    },));
}
