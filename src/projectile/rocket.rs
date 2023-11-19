use rand::prelude::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    collision::PROJECTILE_LAYER,
    turret::{TurretTriggered, TurretType},
    GameAssets, GameState,
};

use super::{Projectile, ProjectileTimer, ProjectileType};

const SPARY_INTENSITY: f32 = 0.05;
const LEFT_TURRET_OFFSET: Vec3 = Vec3::new(5.0, 5.0, 0.0);
const RIGHT_TURRET_OFFSET: Vec3 = Vec3::new(-5.0, 5.0, 0.0);
const DAMAGE: f32 = 5.0;
const LIFE_TIME: f32 = 2.0;

#[derive(Component, Clone)]
pub struct Rocket {
    source_velocity: Vec2,
    current_speed: f32,
}

impl Default for Rocket {
    fn default() -> Self {
        Self {
            source_velocity: Vec2::default(),
            current_speed: 1000.0,
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

fn spawn_rockets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_turret_triggered: EventReader<TurretTriggered>,
) {
    for ev in ev_turret_triggered.read() {
        if ev.turret_type == TurretType::Rocket {
            spawn_rocket(&mut commands, &assets, ev);
        }
    }
}

fn move_rockets(time: Res<Time>, mut rockets: Query<(&mut Transform, &ProjectileTimer, &Rocket)>) {
    for (mut transform, projectile_timer, rocket) in &mut rockets {
        let direction = transform.local_y();
        transform.translation += (direction * rocket.current_speed
            + rocket.source_velocity.extend(0.0))
            * time.delta_seconds();

        let intensity =
            projectile_timer.timer.elapsed_secs() / projectile_timer.timer.duration().as_secs_f32();
        let mut rng = rand::thread_rng();
        transform.rotate_z(rng.gen_range(-1.0..1.0) * intensity.powi(2) * SPARY_INTENSITY);
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
