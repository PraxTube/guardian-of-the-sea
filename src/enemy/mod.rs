use bevy::prelude::*;

use crate::{
    collision::{ENEMY_LAYER, PROJECTILE_LAYER},
    turret::TurretType,
    ui::health::Health,
    vessel::{station::SmallStation1, SpawnVessel},
    GameAssets, GameState,
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

#[derive(Component, Default)]
pub struct Enemy {}

fn spawn_dummy_enemy(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_vessel: EventWriter<SpawnVessel>,
) {
    let transform = Transform::from_translation(Vec3::new(-500.0, 500.0, 0.0));
    let entity = commands
        .spawn((
            Enemy::default(),
            SmallStation1::new(&assets, ENEMY_LAYER, PROJECTILE_LAYER),
        ))
        .insert(transform)
        .id();
    ev_spawn_vessel.send(SpawnVessel {
        entity,
        stats_scale: 1.0,
        turrets: vec![Some(TurretType::MediumRocket)],
        health: Health::new(entity, 1000.0, 2.0),
    });
}

fn despawn_enemies(mut commands: Commands, q_enemies: Query<(Entity, &Health)>) {
    for (entity, health) in &q_enemies {
        if health.health <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
