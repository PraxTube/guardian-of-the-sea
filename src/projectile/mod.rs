mod cannon;
pub mod rocket;
mod rocket_explosion;

use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::GameState;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            cannon::CannonPlugin,
            rocket::RocketPlugin,
            rocket_explosion::RocketExplosionPlugin,
        ))
        .add_event::<ProjectileCollision>()
        .add_event::<ProjectileDespawn>()
        .add_systems(
            Update,
            (
                despawn_projectiles,
                check_projectile_intersections,
                tick_projectile_timers,
            )
                .run_if(in_state(GameState::Gaming)),
        );
    }
}

#[derive(Clone, PartialEq)]
pub enum ProjectileType {
    Cannon,
    Rocket,
}

#[derive(Component, Clone)]
pub struct Projectile {
    pub projectile_type: ProjectileType,
    pub source: Option<Entity>,
    pub damage: f32,
    pub disabled: bool,
}

impl Projectile {
    pub fn new(projectile_type: ProjectileType, source: Entity, damage: f32) -> Self {
        Self {
            projectile_type,
            source: Some(source),
            damage,
            disabled: false,
        }
    }
}

#[derive(Component, Clone)]
pub struct ProjectileTimer {
    pub timer: Timer,
}

impl ProjectileTimer {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(seconds), TimerMode::Once),
        }
    }
}

#[derive(Event)]
pub struct ProjectileCollision {
    pub projectile: Projectile,
    pub target: Entity,
}

#[derive(Event)]
pub struct ProjectileDespawn {
    pub projectile: Projectile,
    pub position: Vec3,
}

fn despawn_projectiles(
    mut commands: Commands,
    q_projectiles: Query<(Entity, &Transform, &Projectile)>,
    mut ev_projectile_despawn: EventWriter<ProjectileDespawn>,
) {
    for (entity, transform, projectile) in &q_projectiles {
        if projectile.disabled {
            commands.entity(entity).despawn_recursive();
            ev_projectile_despawn.send(ProjectileDespawn {
                projectile: projectile.clone(),
                position: transform.translation,
            });
        }
    }
}

fn check_projectile_intersections(
    rapier_context: Res<RapierContext>,
    mut q_projectiles: Query<(Entity, &Transform, &mut Projectile, &Collider)>,
    mut ev_projectile_collision: EventWriter<ProjectileCollision>,
) {
    for (entity, transform, mut projectile, collider) in &mut q_projectiles {
        if projectile.disabled {
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
                if projectile.source == Some(other) {
                    return false;
                }
                ev_projectile_collision.send(ProjectileCollision {
                    projectile: projectile.clone(),
                    target: other,
                });
                projectile.disabled = true;
                false
            },
        );
    }
}

fn tick_projectile_timers(
    time: Res<Time>,
    mut q_projectiles: Query<(&mut Projectile, &mut ProjectileTimer)>,
) {
    for (mut projectile, mut projectile_timer) in &mut q_projectiles {
        projectile_timer.timer.tick(time.delta());

        if projectile_timer.timer.just_finished() {
            projectile.disabled = true;
        }
    }
}
