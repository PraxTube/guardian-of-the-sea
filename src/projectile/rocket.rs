use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    collision::PROJECTILE_LAYER,
    turret::{TurretTriggered, TurretType},
    GameAssets, GameState,
};

use super::{Projectile, ProjectileTimer, ProjectileType};

const LEFT_TURRET_OFFSET: Vec3 = Vec3::new(5.0, 5.0, 0.0);
const RIGHT_TURRET_OFFSET: Vec3 = Vec3::new(-5.0, 5.0, 0.0);
const DAMAGE: f32 = 5.0;
const MEDIUM_DAMAGE: f32 = 20.0;
const LIFE_TIME: f32 = 1.5;
const SPEED: f32 = 750.0;

#[derive(Component, Clone)]
pub struct Rocket {
    source_velocity: Vec2,
    current_speed: f32,
    angle_rotation: f32,
}

impl Default for Rocket {
    fn default() -> Self {
        Self {
            source_velocity: Vec2::default(),
            current_speed: SPEED,
            angle_rotation: 0.0,
        }
    }
}

impl Rocket {
    pub fn new(source_velocity: Vec2) -> Self {
        Self {
            source_velocity,
            ..default()
        }
    }
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
    let rocket = Rocket::new(ev.source_velocity);
    let projectile = Projectile::new(
        ProjectileType::Rocket,
        ev.source,
        ev.turret_mask,
        DAMAGE * ev.stats_scale,
    );
    let projectile_timer = ProjectileTimer::new(LIFE_TIME);
    let collider = Collider::capsule(Vec2::default(), Vec2::new(0.0, 7.0), 4.0);
    let collision_groups = CollisionGroups::new(
        Group::from_bits(PROJECTILE_LAYER).unwrap(),
        Group::from_bits(ev.turret_mask).unwrap(),
    );

    commands.spawn((
        rocket.clone(),
        projectile.clone(),
        projectile_timer.clone(),
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
        projectile.clone(),
        projectile_timer.clone(),
        SpriteBundle {
            transform: right_transform,
            texture: assets.rocket.clone(),
            ..default()
        },
        collider.clone(),
        collision_groups.clone(),
    ));
}

fn spawn_medium_rocket(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    spawn_rotation: Quat,
    life_time: f32,
    angle_rotation: f32,
    ev: &TurretTriggered,
) {
    let transform =
        Transform::from_translation(ev.source_transform.translation).with_rotation(spawn_rotation);
    let rocket = Rocket {
        angle_rotation,
        ..default()
    };
    let projectile = Projectile::new(
        ProjectileType::Rocket,
        ev.source,
        ev.turret_mask,
        MEDIUM_DAMAGE * ev.stats_scale,
    );
    let projectile_timer = ProjectileTimer::new(life_time);
    let collider = Collider::capsule(Vec2::default(), Vec2::new(0.0, 7.0), 4.0);
    let collision_groups = CollisionGroups::new(
        Group::from_bits(PROJECTILE_LAYER).unwrap(),
        Group::from_bits(ev.turret_mask).unwrap(),
    );

    commands.spawn((
        rocket.clone(),
        projectile.clone(),
        projectile_timer.clone(),
        SpriteBundle {
            transform,
            texture: assets.medium_rocket.clone(),
            ..default()
        },
        collider.clone(),
        collision_groups.clone(),
    ));
}

fn spawn_medium_rockets(commands: &mut Commands, assets: &Res<GameAssets>, ev: &TurretTriggered) {
    for i in 0..5 {
        let d = ev
            .source_transform
            .translation
            .truncate()
            .distance(ev.target_point);
        let life_time = d / SPEED;

        let angle = ev.source_transform.rotation.to_euler(EulerRot::ZYX).0;
        let angle_rotation = PI / 4.0 + PI / 16.0 * i as f32;
        let l_rotation = Quat::from_rotation_z(angle + angle_rotation);
        let r_rotation = Quat::from_rotation_z(angle - angle_rotation);

        let angle_rotation = 2.0 * angle_rotation / life_time;
        spawn_medium_rocket(commands, assets, l_rotation, life_time, -angle_rotation, ev);
        spawn_medium_rocket(commands, assets, r_rotation, life_time, angle_rotation, ev);
    }
}

fn spawn_rockets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_turret_triggered: EventReader<TurretTriggered>,
) {
    for ev in ev_turret_triggered.read() {
        if ev.turret_type == TurretType::Rocket {
            spawn_rocket(&mut commands, &assets, ev);
        } else if ev.turret_type == TurretType::MediumRocket {
            spawn_medium_rockets(&mut commands, &assets, ev);
        }
    }
}

fn move_rockets(time: Res<Time>, mut rockets: Query<(&mut Transform, &Rocket)>) {
    for (mut transform, rocket) in &mut rockets {
        let direction = transform.local_y();
        transform.translation += (direction * rocket.current_speed
            + rocket.source_velocity.extend(0.0))
            * time.delta_seconds();

        if rocket.angle_rotation != 0.0 {
            transform.rotate_z(rocket.angle_rotation * time.delta_seconds());
        }
    }
}

pub struct RocketPlugin;

impl Plugin for RocketPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_rockets, move_rockets).run_if(in_state(GameState::Gaming)),
        );
    }
}
