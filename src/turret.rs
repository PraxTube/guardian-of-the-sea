use bevy::prelude::*;

use crate::utils::quat_from_vec2;
use crate::{GameAssets, MouseWorldCoords, Player};

const TURRET_Z_OFFSET: Vec3 = Vec3::new(0.0, 0.0, 10.0);

#[derive(Component, Clone)]
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

impl Turret {
    pub fn new(offset: Vec2) -> Self {
        Self {
            offset: offset.extend(0.0),
        }
    }
}

#[derive(Event)]
pub struct SpawnTurretsEvent {
    pub turrets: Vec<Turret>,
}

pub fn spawn_turrets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventReader<SpawnTurretsEvent>,
) {
    for ev in ev_spawn_turrets.read() {
        for turret in &ev.turrets {
            commands.spawn((
                SpriteBundle {
                    texture: assets.turret.clone(),
                    ..default()
                },
                turret.clone(),
            ));
        }
    }
}

pub fn reposition_turrets(
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

pub fn rotate_turrets(
    mut turrets: Query<&mut Transform, With<Turret>>,
    mouse_coords: Res<MouseWorldCoords>,
) {
    for mut turret in &mut turrets {
        turret.rotation =
            quat_from_vec2(-1.0 * (mouse_coords.0 - turret.translation.truncate()).perp());
    }
}
