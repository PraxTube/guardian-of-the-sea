use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    player::Player,
    turret::{SpawnTurretsEvent, Turret},
    ui::health::{Health, SpawnHealth},
    utils::quat_from_vec2,
    GameAssets, GameState, ShipStats,
};

const MIN_ANGLE_THRESHOLD: f32 = 0.08;

pub struct GuardianEnemyPlugin;

impl Plugin for GuardianEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_enemies, update_enemy_move_targets, steer_enemies)
                .run_if(in_state(GameState::Gaming)),
        )
        .add_systems(OnEnter(GameState::Gaming), spawn_dummy_enemy);
    }
}

#[derive(Component, Default)]
pub struct Enemy {
    pub target_point: Vec2,
}

fn spawn_dummy_enemy(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventWriter<SpawnTurretsEvent>,
    mut ev_spawn_health: EventWriter<SpawnHealth>,
) {
    let transform = Transform::from_translation(Vec3::new(500.0, 500.0, 0.0));
    let collider = Collider::capsule(Vec2::new(0.0, -20.0), Vec2::new(0.0, 20.0), 15.0);
    let mut ship_stats = ShipStats::default();
    ship_stats.current_speed = 50.0;

    let entity = commands
        .spawn((
            Enemy::default(),
            ship_stats.clone(),
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

fn update_enemy_move_targets(
    mut q_enemies: Query<(&mut Transform, &ShipStats, &mut Enemy), Without<Player>>,
    q_player: Query<(&Transform, &ShipStats), (With<Player>, Without<Enemy>)>,
) {
    let (player_transform, player_ship_stats) = match q_player.get_single() {
        Ok(p) => (p.0, p.1),
        Err(_) => return,
    };

    let p = player_transform.translation.truncate();
    let p_perp = player_transform.local_y().truncate().perp() * 500.0;

    for (mut enemy_transform, enemy_ship_stats, mut enemy) in &mut q_enemies {
        let e = enemy_transform.translation.truncate();
        let l_target = p + p_perp - e;
        let r_target = p - p_perp - e;

        let target = if l_target.length_squared() < r_target.length_squared() {
            l_target
        } else {
            r_target
        };
        enemy.target_point = target;
    }
}

fn steer_enemies(time: Res<Time>, mut q_enemies: Query<(&mut Transform, &ShipStats, &Enemy)>) {
    for (mut transform, ship_stats, enemy) in &mut q_enemies {
        let angle = enemy
            .target_point
            .angle_between(transform.local_y().truncate());
        if angle.abs() <= MIN_ANGLE_THRESHOLD {
            continue;
        }

        let steer_direction = if angle < 0.0 { 1.0 } else { -1.0 };

        let rotation = ship_stats.delta_steering * steer_direction * time.delta_seconds();
        transform.rotate_z(rotation);
    }
}
