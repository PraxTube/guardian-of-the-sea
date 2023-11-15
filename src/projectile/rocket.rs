use std::time::Duration;

use rand::prelude::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

//use crate::turret::Turret;
use crate::{turret::Turret, GameAssets};

const SPARY_INTENSITY: f32 = 0.05;

#[derive(Component, Clone)]
pub struct Rocket {
    current_speed: f32,
    timer: Timer,
}

impl Default for Rocket {
    fn default() -> Self {
        Self {
            current_speed: 0.0,
            timer: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once),
        }
    }
}

#[derive(Event)]
pub struct RocketFired {
    rocket_turret: RocketTurret,
}

pub struct RocketTurret {
    pub spawn_point: Vec3,
    pub spawn_rotation: Quat,
    pub left_offset: Vec3,
    pub right_offset: Vec3,
    pub speed: f32,
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
    let rocket = Rocket {
        current_speed: ev.rocket_turret.speed,
        ..default()
    };
    let collider = Collider::capsule(Vec2::default(), Vec2::new(0.0, 50.0), 50.0);

    commands.spawn((
        rocket.clone(),
        SpriteBundle {
            transform: left_transform,
            texture: assets.rocket.clone(),
            ..default()
        },
        collider.clone(),
    ));

    commands.spawn((
        rocket.clone(),
        SpriteBundle {
            transform: right_transform,
            texture: assets.rocket.clone(),
            ..default()
        },
        collider.clone(),
    ));
}

pub fn fire_rockets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_rocket_fired: EventReader<RocketFired>,
) {
    for ev in ev_rocket_fired.read() {
        spawn_rocket(&mut commands, &assets, ev);
    }
}

pub fn move_rockets(time: Res<Time>, mut rockets: Query<(&mut Transform, &Rocket)>) {
    for (mut transform, rocket) in &mut rockets {
        let direction = transform.local_y();
        transform.translation += direction * rocket.current_speed * time.delta_seconds();

        let intensity = rocket.timer.elapsed_secs() / rocket.timer.duration().as_secs_f32();
        let mut rng = rand::thread_rng();
        transform.rotate_z(rng.gen_range(-1.0..1.0) * intensity.powi(2) * SPARY_INTENSITY);
    }
}

pub fn shoot_rockets(
    keys: Res<Input<KeyCode>>,
    mut q_turrets: Query<(&mut Turret, &Transform)>,
    mut ev_rocket_fired: EventWriter<RocketFired>,
) {
    if !keys.pressed(KeyCode::Space) {
        return;
    }

    for (mut turret, transform) in &mut q_turrets {
        if turret.cooling_down {
            continue;
        }
        turret.cooling_down = true;

        ev_rocket_fired.send(RocketFired {
            rocket_turret: RocketTurret {
                spawn_point: transform.translation,
                spawn_rotation: transform.rotation,
                left_offset: Vec3::new(5.0, 5.0, 0.0),
                right_offset: Vec3::new(-5.0, 5.0, 0.0),
                speed: 1000.0,
            },
        })
    }
}

pub fn despawn_rockets(
    mut commands: Commands,
    time: Res<Time>,
    mut q_rockets: Query<(Entity, &mut Rocket)>,
) {
    for (entity, mut rocket) in &mut q_rockets {
        rocket.timer.tick(time.delta());

        if rocket.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn test_intersections(
    rapier_context: Res<RapierContext>,
    q_rockets: Query<(&Transform, &Rocket, &Collider)>,
) {
    for (transform, rocket, collider) in &q_rockets {
        let filter = QueryFilter::default();

        rapier_context.intersections_with_shape(
            transform.translation.truncate(),
            transform.rotation.to_euler(EulerRot::ZYX).0,
            collider,
            filter,
            |entity| {
                println!(
                    "The entity {:?} intersects our shape.",
                    q_rockets.get(entity).unwrap().1.current_speed
                );
                true // Return `false` instead if we want to stop searching for other colliders that contain this point.
            },
        );
    }
}
