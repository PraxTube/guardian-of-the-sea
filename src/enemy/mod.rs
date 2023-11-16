use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    turret::{SpawnTurretsEvent, Turret},
    ui::health::{Health, SpawnHealth},
    GameAssets, GameState, ShipStats,
};

pub struct GuardianEnemyPlugin;

impl Plugin for GuardianEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_enemies).run_if(in_state(GameState::Gaming)),
        )
        .add_systems(OnEnter(GameState::Gaming), spawn_dummy_enemy);
    }
}

#[derive(Component)]
pub struct Enemy;

fn spawn_dummy_enemy(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventWriter<SpawnTurretsEvent>,
    mut ev_spawn_health: EventWriter<SpawnHealth>,
) {
    let transform = Transform::from_translation(Vec3::new(500.0, 500.0, 0.0));
    let collider = Collider::capsule(Vec2::new(0.0, -20.0), Vec2::new(0.0, 20.0), 15.0);
    let ship_stats = ShipStats::default();

    let entity = commands
        .spawn((
            Enemy,
            collider,
            CollisionGroups::new(
                Group::from_bits(0b0100).unwrap(),
                Group::from_bits(0b1000).unwrap(),
            ),
            SpriteBundle {
                transform,
                texture: assets.dummy_enemy.clone(),
                ..default()
            },
        ))
        .id();
    ev_spawn_turrets.send(SpawnTurretsEvent {
        turrets: vec![Turret::new(entity, Vec2::default())],
    });
    ev_spawn_health.send(SpawnHealth {
        entity,
        health: Health::new(entity, 100.0, ship_stats),
    })
}

fn despawn_enemies(mut commands: Commands, q_enemies: Query<(Entity, &Health)>) {
    for (entity, health) in &q_enemies {
        if health.health <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn aim_enemy_turrets() {}

fn shoot_enemy_turrets() {}
