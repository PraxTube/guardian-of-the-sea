use std::time::Duration;

use bevy::prelude::*;

use crate::player::input::{fetch_mouse_world_coords, MouseWorldCoords};
use crate::utils::quat_from_vec2;
use crate::{move_ships, GameAssets, GameState};

const TURRET_Z_OFFSET: Vec3 = Vec3::new(0.0, 0.0, 10.0);

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_turrets,
                reposition_turrets.after(move_ships),
                rotate_turrets.after(fetch_mouse_world_coords),
                cooldown_turrets,
            )
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<SpawnTurretsEvent>();
    }
}

#[derive(Component, Clone)]
pub struct Turret {
    pub source: Option<Entity>,
    pub offset: Vec3,
    pub cooling_down: bool,
    pub cooldown_timer: Timer,
}

impl Default for Turret {
    fn default() -> Self {
        Self {
            source: None,
            offset: Vec3::default(),
            cooling_down: false,
            cooldown_timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Repeating),
        }
    }
}

impl Turret {
    pub fn new(source: Entity, offset: Vec2) -> Self {
        Self {
            source: Some(source),
            offset: offset.extend(0.0),
            ..default()
        }
    }
}

#[derive(Event)]
pub struct SpawnTurretsEvent {
    pub turrets: Vec<Turret>,
}

fn spawn_turrets(
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

fn reposition_turrets(
    mut q_turrets: Query<(&mut Transform, &Turret)>,
    q_transforms: Query<&Transform, Without<Turret>>,
) {
    for (mut turret_transform, turret) in &mut q_turrets {
        let source = match turret.source {
            Some(s) => s,
            None => continue,
        };
        let source_transform = match q_transforms.get(source) {
            Ok(t) => t,
            Err(_) => continue,
        };
        turret_transform.translation = source_transform.translation
            + source_transform.rotation.mul_vec3(turret.offset)
            + TURRET_Z_OFFSET;
    }
}

fn rotate_turrets(
    mut turrets: Query<&mut Transform, With<Turret>>,
    mouse_coords: Res<MouseWorldCoords>,
) {
    for mut turret in &mut turrets {
        turret.rotation =
            quat_from_vec2(-1.0 * (mouse_coords.0 - turret.translation.truncate()).perp());
    }
}

fn cooldown_turrets(time: Res<Time>, mut q_turrets: Query<&mut Turret>) {
    for mut turret in &mut q_turrets {
        if !turret.cooling_down {
            continue;
        }

        turret.cooldown_timer.tick(time.delta());

        if turret.cooldown_timer.just_finished() {
            turret.cooling_down = false;
        }
    }
}
