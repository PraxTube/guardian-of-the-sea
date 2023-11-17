use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    turret::{TurretTriggered, TurretType},
    GameAssets, GameState,
};

#[derive(Component, Clone)]
pub struct Cannon {
    source: Option<Entity>,
    source_velocity: Vec2,
    current_speed: f32,
    pub damage: f32,
    timer: Timer,
    disabled: bool,
}

impl Default for Cannon {
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

impl Cannon {
    pub fn new(source: Entity, source_velocity: Vec2) -> Self {
        Self {
            source: Some(source),
            source_velocity,
            ..default()
        }
    }
}

#[derive(Event)]
pub struct CannonCollision {
    pub entity: Entity,
    pub rocket: Cannon,
}

#[derive(Event)]
pub struct CannonDespawn {
    pub position: Vec3,
}

fn spawn_cannon(commands: &mut Commands, assets: &Res<GameAssets>, ev: &TurretTriggered) {
    let transform = Transform::from_translation(ev.source_transform.translation)
        .with_rotation(ev.source_transform.rotation);
    let cannon = Cannon::new(ev.source, ev.source_velocity);
    let collider = Collider::capsule(Vec2::default(), Vec2::new(0.0, 7.0), 4.0);
    let collision_groups = CollisionGroups::new(
        Group::from_bits(0b1000).unwrap(),
        Group::from_bits(0b0100).unwrap(),
    );

    commands.spawn((
        cannon,
        SpriteBundle {
            transform,
            texture: assets.cannon.clone(),
            ..default()
        },
        collider,
        collision_groups,
    ));
}

fn spawn_cannons(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_turret_triggered: EventReader<TurretTriggered>,
) {
    for ev in ev_turret_triggered.read() {
        if ev.turret_type == TurretType::Cannon {
            spawn_cannon(&mut commands, &assets, ev);
        }
    }
}

fn move_cannons(time: Res<Time>, mut q_cannons: Query<(&mut Transform, &Cannon)>) {
    for (mut transform, cannon) in &mut q_cannons {
        let direction = transform.local_y();
        transform.translation += (direction * cannon.current_speed
            + cannon.source_velocity.extend(0.0))
            * time.delta_seconds();
    }
}

fn despawn_cannons(
    mut commands: Commands,
    time: Res<Time>,
    mut q_cannons: Query<(Entity, &Transform, &mut Cannon)>,
    mut ev_cannon_despawn: EventWriter<CannonDespawn>,
) {
    for (entity, transform, mut cannon) in &mut q_cannons {
        cannon.timer.tick(time.delta());

        if cannon.timer.just_finished() || cannon.disabled {
            commands.entity(entity).despawn_recursive();
            ev_cannon_despawn.send(CannonDespawn {
                position: transform.translation,
            });
        }
    }
}

fn check_cannon_collisions(
    rapier_context: Res<RapierContext>,
    mut q_cannons: Query<(Entity, &Transform, &mut Cannon, &Collider)>,
    mut ev_rocket_collision: EventWriter<CannonCollision>,
) {
    for (entity, transform, mut cannon, collider) in &mut q_cannons {
        if cannon.disabled {
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
                if let Some(source) = cannon.source {
                    if source == other {
                        return false;
                    }
                }
                ev_rocket_collision.send(CannonCollision {
                    entity: other,
                    rocket: cannon.clone(),
                });
                cannon.disabled = true;
                false
            },
        );
    }
}

pub struct CannonPlugin;

impl Plugin for CannonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_cannons,
                move_cannons,
                despawn_cannons,
                check_cannon_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<CannonCollision>()
        .add_event::<CannonDespawn>();
    }
}
