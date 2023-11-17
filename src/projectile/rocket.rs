use std::time::Duration;

use rand::prelude::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    turret::{TurretTriggered, TurretType},
    GameAssets, GameState,
};

const SPARY_INTENSITY: f32 = 0.05;
const LEFT_TURRET_OFFSET: Vec3 = Vec3::new(5.0, 5.0, 0.0);
const RIGHT_TURRET_OFFSET: Vec3 = Vec3::new(-5.0, 5.0, 0.0);

#[derive(Component, Clone)]
pub struct Rocket {
    source: Option<Entity>,
    source_velocity: Vec2,
    current_speed: f32,
    pub damage: f32,
    timer: Timer,
    disabled: bool,
}

impl Default for Rocket {
    fn default() -> Self {
        Self {
            source: None,
            source_velocity: Vec2::default(),
            current_speed: 1000.0,
            damage: 1.0,
            timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Once),
            disabled: false,
        }
    }
}

impl Rocket {
    pub fn new(source: Entity, source_velocity: Vec2) -> Self {
        Self {
            source: Some(source),
            source_velocity,
            ..default()
        }
    }
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

fn spawn_rocket(commands: &mut Commands, assets: &Res<GameAssets>, ev: &TurretTriggered) {
    let left_transform = Transform::from_translation(
        ev.source_transform.translation + ev.source_transform.rotation.mul_vec3(LEFT_TURRET_OFFSET),
    )
    .with_rotation(ev.source_transform.rotation);
    let right_transform = Transform::from_translation(
        ev.source_transform.translation
            + ev.source_transform.rotation.mul_vec3(RIGHT_TURRET_OFFSET),
    )
    .with_rotation(ev.source_transform.rotation);
    let rocket = Rocket::new(ev.source, ev.source_velocity);
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

fn spawn_rockets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_rocket_fired: EventReader<TurretTriggered>,
) {
    for ev in ev_rocket_fired.read() {
        if ev.turret_type == TurretType::Rocket {
            spawn_rocket(&mut commands, &assets, ev);
        }
    }
}

fn move_rockets(time: Res<Time>, mut rockets: Query<(&mut Transform, &Rocket)>) {
    for (mut transform, rocket) in &mut rockets {
        let direction = transform.local_y();
        transform.translation += (direction * rocket.current_speed
            + rocket.source_velocity.extend(0.0))
            * time.delta_seconds();

        let intensity = rocket.timer.elapsed_secs() / rocket.timer.duration().as_secs_f32();
        let mut rng = rand::thread_rng();
        transform.rotate_z(rng.gen_range(-1.0..1.0) * intensity.powi(2) * SPARY_INTENSITY);
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
                spawn_rockets,
                move_rockets,
                despawn_rockets,
                check_rocket_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<RocketCollision>()
        .add_event::<RocketDespawn>();
    }
}
