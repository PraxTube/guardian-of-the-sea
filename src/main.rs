use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowMode};

use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

mod enemy;
mod player;
mod projectile;
mod turret;
mod ui;
mod utils;
mod world;

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

    #[asset(path = "dummy_enemy.png")]
    pub dummy_enemy: Handle<Image>,

    #[asset(path = "turret.png")]
    pub turret: Handle<Image>,

    #[asset(path = "water.png")]
    pub water: Handle<Image>,

    #[asset(path = "rocket.png")]
    pub rocket: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 32.0, tile_size_y = 32.0, columns = 8, rows = 1))]
    #[asset(path = "gfx/explosion.png")]
    pub explosion: Handle<TextureAtlas>,
}

#[derive(Component, Clone)]
pub struct ShipStats {
    delta_steering: f32,
    delta_speed: f32,
    current_speed: f32,

    min_speed: f32,
    max_speed: f32,

    health_bar_offset: Vec3,
    health_bar_scale: Vec3,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            delta_steering: 4.0,
            delta_speed: 100.0,
            current_speed: 0.0,
            min_speed: -150.0,
            max_speed: 500.0,
            health_bar_offset: Vec3::new(-30.0, -40.0, 0.0),
            health_bar_scale: Vec3::new(60.0, 7.5, 1.0),
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
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .build(),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((
            world::GuardianWorldPlugin,
            ui::GuardianUiPlugin,
            projectile::ProjectilePlugin,
            turret::TurretPlugin,
            enemy::GuardianEnemyPlugin,
            player::GuardianPlayerPlugin,
        ))
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .add_systems(OnEnter(GameState::Gaming), (spawn_water_tiles,))
        .add_systems(Update, (move_ships,).run_if(in_state(GameState::Gaming)))
        .run();
}

fn move_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &ShipStats)>) {
    for (mut transform, ship_stats) in &mut ships {
        let direction = transform.local_y();
        transform.translation += direction * ship_stats.current_speed * time.delta_seconds();
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
