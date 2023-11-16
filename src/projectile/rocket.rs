use std::time::Duration;

use rand::prelude::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{enemy::Enemy, player::Player, turret::Turret, GameAssets, GameState};

const SPARY_INTENSITY: f32 = 0.05;

#[derive(Component, Clone)]
pub struct Rocket {
    source: Option<Entity>,
    current_speed: f32,
    pub damage: f32,
    timer: Timer,
    disabled: bool,
}

impl Default for Rocket {
    fn default() -> Self {
        Self {
            source: None,
            current_speed: 0.0,
            damage: 1.0,
            timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Once),
            disabled: false,
        }
    }
}

impl Rocket {
    pub fn new(source: Entity, current_speed: f32) -> Self {
        Self {
            source: Some(source),
            current_speed,
            ..default()
        }
    }
}

pub struct RocketTurret {
    pub spawn_point: Vec3,
    pub spawn_rotation: Quat,
    pub left_offset: Vec3,
    pub right_offset: Vec3,
    pub speed: f32,
}

#[derive(Event)]
pub struct RocketFired {
    source: Entity,
    rocket_turret: RocketTurret,
}

#[derive(Event)]
pub struct RocketCollision {
    pub entity: Entity,
    pub rocket: Rocket,
}

#[derive(Event)]
pub struct RocketDespawn {
    pub position: Vec3,
}

fn spawn_rocket(commands: &mut Commands, assets: &Res<GameAssets>, ev: &RocketFired) {
    let left_transform = Transform::from_translation(
        ev.rocket_turret.spawn_point
            + ev.rocket_turret
                .spawn_rotation
                .mul_vec3(ev.rocket_turret.left_offset),
    )
    .with_rotation(ev.rocket_turret.spawn_rotation);
    let right_transform = Transform::from_translation(
        ev.rocket_turret.spawn_point
            + ev.rocket_turret
                .spawn_rotation
                .mul_vec3(ev.rocket_turret.right_offset),
    )
    .with_rotation(ev.rocket_turret.spawn_rotation);
    let rocket = Rocket::new(ev.source, ev.rocket_turret.speed);
    let collider = Collider::capsule(Vec2::default(), Vec2::new(0.0, 7.0), 4.0);
    let collision_groups = CollisionGroups::new(
        Group::from_bits(0b1000).unwrap(),
        Group::from_bits(0b0100).unwrap(),
    );

    commands.spawn((
        rocket.clone(),
        SpriteBundle {
            transform: left_transform,
            texture: assets.rocket.clone(),
            ..default()
        },
        collider.clone(),
        collision_groups.clone(),
    ));

    commands.spawn((
        rocket.clone(),
        SpriteBundle {
            transform: right_transform,
            texture: assets.rocket.clone(),
            ..default()
        },
        collider.clone(),
        collision_groups.clone(),
    ));
}

fn fire_rockets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_rocket_fired: EventReader<RocketFired>,
) {
    for ev in ev_rocket_fired.read() {
        spawn_rocket(&mut commands, &assets, ev);
    }
}

fn move_rockets(time: Res<Time>, mut rockets: Query<(&mut Transform, &Rocket)>) {
    for (mut transform, rocket) in &mut rockets {
        let direction = transform.local_y();
        transform.translation += direction * rocket.current_speed * time.delta_seconds();

        let intensity = rocket.timer.elapsed_secs() / rocket.timer.duration().as_secs_f32();
        let mut rng = rand::thread_rng();
        transform.rotate_z(rng.gen_range(-1.0..1.0) * intensity.powi(2) * SPARY_INTENSITY);
    }
}

fn shoot_player_rockets(
    keys: Res<Input<KeyCode>>,
    mut q_turrets: Query<(&mut Turret, &Transform)>,
    q_player: Query<Entity, With<Player>>,
    mut ev_rocket_fired: EventWriter<RocketFired>,
) {
    if !keys.pressed(KeyCode::Space) {
        return;
    }

    let player = match q_player.get_single() {
        Ok(p) => p,
        Err(err) => {
            error!("there should be exactly on player, {}", err);
            return;
        }
    };

    for (mut turret, transform) in &mut q_turrets {
        if turret.cooling_down {
            continue;
        }

        let source = match turret.source {
            Some(s) => s,
            None => {
                error!("the shooting turret does not have a source");
                continue;
            }
        };

        if source != player {
            continue;
        }

        ev_rocket_fired.send(RocketFired {
            source,
            rocket_turret: RocketTurret {
                spawn_point: transform.translation,
                spawn_rotation: transform.rotation,
                left_offset: Vec3::new(5.0, 5.0, 0.0),
                right_offset: Vec3::new(-5.0, 5.0, 0.0),
                speed: 1000.0,
            },
        });
        turret.cooling_down = true;
    }
}

fn shoot_enemy_rockets(
    mut q_turrets: Query<(&mut Turret, &Transform)>,
    q_enemies: Query<Entity, With<Enemy>>,
    mut ev_rocket_fired: EventWriter<RocketFired>,
) {
    for (mut turret, transform) in &mut q_turrets {
        if turret.cooling_down {
            continue;
        }

        let source = match turret.source {
            Some(s) => s,
            None => {
                error!("the shooting turret does not have a source");
                continue;
            }
        };

        if q_enemies.get(source).is_err() {
            continue;
        }

        ev_rocket_fired.send(RocketFired {
            source,
            rocket_turret: RocketTurret {
                spawn_point: transform.translation,
                spawn_rotation: transform.rotation,
                left_offset: Vec3::new(5.0, 5.0, 0.0),
                right_offset: Vec3::new(-5.0, 5.0, 0.0),
                speed: 1000.0,
            },
        });
        turret.cooling_down = true;
    }
}

fn despawn_rockets(
    mut commands: Commands,
    time: Res<Time>,
    mut q_rockets: Query<(Entity, &Transform, &mut Rocket)>,
    mut ev_rocket_despawn: EventWriter<RocketDespawn>,
) {
    for (entity, transform, mut rocket) in &mut q_rockets {
        rocket.timer.tick(time.delta());

        if rocket.timer.just_finished() || rocket.disabled {
            commands.entity(entity).despawn_recursive();
            ev_rocket_despawn.send(RocketDespawn {
                position: transform.translation,
            });
        }
    }
}

fn check_rocket_collisions(
    rapier_context: Res<RapierContext>,
    mut q_rockets: Query<(Entity, &Transform, &mut Rocket, &Collider)>,
    mut ev_rocket_collision: EventWriter<RocketCollision>,
) {
    for (entity, transform, mut rocket, collider) in &mut q_rockets {
        if rocket.disabled {
            continue;
        }

        let filter = QueryFilter {
            groups: Some(CollisionGroups::new(
                Group::from_bits(0b1000).unwrap(),
                Group::from_bits(0b0100).unwrap(),
            )),
            exclude_collider: Some(entity),
            ..default()
        };

        rapier_context.intersections_with_shape(
            transform.translation.truncate(),
            transform.rotation.to_euler(EulerRot::ZYX).0,
            collider,
            filter,
            |other| {
                if let Some(source) = rocket.source {
                    if source == other {
                        return false;
                    }
                }
                ev_rocket_collision.send(RocketCollision {
                    entity: other,
                    rocket: rocket.clone(),
                });
                rocket.disabled = true;
                false
            },
        );
    }
}

pub struct RocketPlugin;

impl Plugin for RocketPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                shoot_player_rockets,
                shoot_enemy_rockets,
                fire_rockets,
                move_rockets,
                despawn_rockets,
                check_rocket_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<RocketFired>()
        .add_event::<RocketCollision>()
        .add_event::<RocketDespawn>();
    }
}
