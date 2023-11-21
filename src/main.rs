use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowMode};

use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

mod assets;
mod collision;
mod enemy;
mod player;
mod projectile;
mod turret;
mod ui;
mod utils;
mod vessel;
mod world;

pub use assets::GameAssets;
pub use vessel::ship::ShipStats;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    Gaming,
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
            utils::GuardianUtilsPlugin,
            projectile::ProjectilePlugin,
            turret::TurretPlugin,
            vessel::GuardianVesselPlugin,
            enemy::GuardianEnemyPlugin,
            player::GuardianPlayerPlugin,
        ))
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .add_systems(OnEnter(GameState::Gaming), spawn_water_tiles)
        .run();
}

fn spawn_water_tiles(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((SpriteBundle {
        texture: assets.water.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -100.0))
            .with_scale(Vec3::splat(10.0)),
        ..default()
    },));
}
