use std::time::Duration;

use bevy::prelude::*;

use crate::collision::{ENEMY_LAYER, PLAYER_LAYER};
use crate::enemy::Enemy;
use crate::player::input::{fetch_mouse_world_coords, MouseWorldCoords};
use crate::player::Player;
use crate::utils::quat_from_vec2;
use crate::vessel::ship::{move_ships, steer_ships};
use crate::vessel::SpawnVessel;
use crate::{GameAssets, GameState, ShipStats};

const TURRET_Z_OFFSET: Vec3 = Vec3::new(0.0, 0.0, 10.0);

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                reposition_turrets.after(move_ships).after(steer_ships),
                update_player_turret_targets,
                update_enemy_turret_targets,
                rotate_turrets.after(fetch_mouse_world_coords),
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<TurretTriggered>()
        .add_systems(
            Update,
            (
                spawn_turrets,
                cooldown_turrets,
                despawn_turrets,
                trigger_player_turrets,
                trigger_enemy_turrets,
            )
                .run_if(in_state(GameState::Gaming)),
        );
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TurretType {
    Cannon,
    Rocket,
    MediumRocket,
}

#[derive(Component, Clone)]
pub struct Turret {
    pub turret_type: TurretType,
    pub stats_scale: f32,
    pub source: Entity,
    pub target_direction: Vec2,
    pub offset: Vec3,
    pub cooling_down: bool,
    pub cooldown_timer: Timer,
}

impl Turret {
    pub fn new(turret_type: TurretType, stats_scale: f32, source: Entity, offset: Vec2) -> Self {
        Self {
            turret_type,
            stats_scale,
            source,
            target_direction: Vec2::default(),
            offset: offset.extend(0.0),
            cooling_down: false,
            cooldown_timer: Timer::new(
                Duration::from_secs_f32(cooldown_from_turret_type(turret_type, stats_scale)),
                TimerMode::Repeating,
            ),
        }
    }
}

#[derive(Component, Clone)]
pub struct TurretStats {
    pub turret_offsets: Vec<Vec2>,
}

#[derive(Event)]
pub struct TurretTriggered {
    pub turret_type: TurretType,
    pub turret_layer: u32,
    pub turret_mask: u32,
    pub source: Entity,
    pub source_transform: Transform,
    pub source_velocity: Vec2,
    pub stats_scale: f32,
}

fn cooldown_from_turret_type(turret_type: TurretType, stats_scale: f32) -> f32 {
    match turret_type {
        TurretType::Cannon => 0.1 / stats_scale,
        TurretType::Rocket => 0.5 / stats_scale,
        TurretType::MediumRocket => 1.0 / stats_scale,
    }
}

fn spawn_turrets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    q_turret_stats: Query<&TurretStats>,
    mut ev_spawn_turrets: EventReader<SpawnVessel>,
) {
    for ev in ev_spawn_turrets.read() {
        for (i, turret) in ev.turrets.iter().enumerate() {
            let turret_type = match turret {
                Some(t) => t,
                None => continue,
            };
            let turret_stats = match q_turret_stats.get(ev.entity) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let texture = match turret_type {
                TurretType::Cannon => assets.cannon_turret.clone(),
                TurretType::Rocket => assets.rocket_turret.clone(),
                TurretType::MediumRocket => assets.medium_rocket_turret.clone(),
            };
            commands.spawn((
                SpriteBundle {
                    texture,
                    ..default()
                },
                Turret::new(
                    turret_type.clone(),
                    ev.stats_scale,
                    ev.entity,
                    turret_stats.turret_offsets[i],
                ),
            ));
        }
    }
}

fn reposition_turrets(
    mut q_turrets: Query<(&mut Transform, &Turret)>,
    q_transforms: Query<&Transform, Without<Turret>>,
) {
    for (mut turret_transform, turret) in &mut q_turrets {
        let source_transform = match q_transforms.get(turret.source) {
            Ok(t) => t,
            Err(_) => continue,
        };
        turret_transform.translation = source_transform.translation
            + source_transform.rotation.mul_vec3(turret.offset)
            + TURRET_Z_OFFSET;
    }
}

fn rotate_turrets(mut turrets: Query<(&mut Transform, &Turret)>) {
    for (mut transform, turret) in &mut turrets {
        transform.rotation = quat_from_vec2(turret.target_direction);
    }
}

fn update_player_turret_targets(
    mut turrets: Query<(&Transform, &mut Turret)>,
    q_player: Query<Entity, With<Player>>,
    mouse_coords: Res<MouseWorldCoords>,
) {
    let player = match q_player.get_single() {
        Ok(p) => p,
        Err(err) => {
            error!("not exactly one player, {}", err);
            return;
        }
    };

    for (transform, mut turret) in &mut turrets {
        if turret.source != player {
            continue;
        }
        turret.target_direction = -1.0 * (mouse_coords.0 - transform.translation.truncate()).perp();
    }
}

fn update_enemy_turret_targets(
    mut turrets: Query<(&Transform, &mut Turret)>,
    q_player: Query<&Transform, With<Player>>,
    q_enemies: Query<Entity, With<Enemy>>,
) {
    let player_transform = match q_player.get_single() {
        Ok(p) => p,
        Err(err) => {
            error!("not exactly one player, {}", err);
            return;
        }
    };

    for (transform, mut turret) in &mut turrets {
        if q_enemies.get(turret.source).is_err() {
            continue;
        }
        turret.target_direction = -1.0
            * (player_transform.translation.truncate() - transform.translation.truncate()).perp();
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

fn despawn_turrets(
    mut commands: Commands,
    q_transforms: Query<&Transform, Without<Turret>>,
    q_turrets: Query<(Entity, &Turret)>,
) {
    for (entity, turret) in &q_turrets {
        if q_transforms.get(turret.source).is_err() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn trigger_player_turrets(
    keys: Res<Input<KeyCode>>,
    mut q_turrets: Query<(&mut Turret, &Transform)>,
    q_player: Query<(Entity, &Transform, &ShipStats), With<Player>>,
    mut ev_rocket_fired: EventWriter<TurretTriggered>,
) {
    if !keys.pressed(KeyCode::Space) {
        return;
    }

    let (player, p_transform, ship_stats) = match q_player.get_single() {
        Ok(p) => (p.0, p.1, p.2),
        Err(err) => {
            error!("there should be exactly on player, {}", err);
            return;
        }
    };

    for (mut turret, transform) in &mut q_turrets {
        if turret.cooling_down {
            continue;
        }

        if turret.source != player {
            continue;
        }

        ev_rocket_fired.send(TurretTriggered {
            turret_type: turret.turret_type,
            turret_layer: PLAYER_LAYER,
            turret_mask: ENEMY_LAYER,
            source: turret.source,
            source_transform: transform.clone(),
            source_velocity: p_transform.local_y().truncate() * ship_stats.current_speed,
            stats_scale: turret.stats_scale,
        });
        turret.cooling_down = true;
    }
}

fn trigger_enemy_turrets(
    mut q_turrets: Query<(&mut Turret, &Transform)>,
    q_enemies: Query<&Transform, With<Enemy>>,
    mut ev_rocket_fired: EventWriter<TurretTriggered>,
) {
    for (mut turret, transform) in &mut q_turrets {
        if turret.cooling_down {
            continue;
        }

        let e_transform = match q_enemies.get(turret.source) {
            Ok(t) => t,
            Err(_) => continue,
        };

        ev_rocket_fired.send(TurretTriggered {
            turret_type: turret.turret_type,
            turret_layer: ENEMY_LAYER,
            turret_mask: PLAYER_LAYER,
            source: turret.source,
            source_transform: transform.clone(),
            source_velocity: e_transform.local_y().truncate(),
            stats_scale: turret.stats_scale,
        });
        turret.cooling_down = true;
    }
}
