use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::{move_ships, GameState, MouseWorldCoords, Player};

pub struct GuardianCameraPlugin;

impl Plugin for GuardianCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gaming), (spawn_camera,))
            .add_systems(
                Update,
                move_camera
                    .after(move_ships)
                    .run_if(in_state(GameState::Gaming)),
            );
    }
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(750.0);
    commands.spawn((MainCamera, camera));
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

    camera_transform.translation =
        player_pos + (mouse_coords.0.extend(0.0) - player_pos) / 4.0 / projection.scale;
}
