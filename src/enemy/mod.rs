use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    player::Player,
    turret::TurretType,
    ui::health::Health,
    vessel::{ship::SmallShip1, SpawnVessel},
    GameAssets, GameState, ShipStats,
};

const MIN_ANGLE_THRESHOLD: f32 = 0.08;
const MIN_PLAYER_DISTANCE: f32 = 300.0;
const MAX_DISTANCE_SQUARED: f32 = 100_000.0;

pub struct GuardianEnemyPlugin;

impl Plugin for GuardianEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_enemies,
                update_enemy_move_targets,
                steer_enemies,
                accelerate_enemies,
            )
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
    mut ev_spawn_vessel: EventWriter<SpawnVessel>,
) {
    let transform = Transform::from_translation(Vec3::new(500.0, 500.0, 0.0));
    let entity = commands
        .spawn((
            Enemy::default(),
            SmallShip1::new(&assets),
            CollisionGroups::new(
                Group::from_bits(0b0100).unwrap(),
                Group::from_bits(0b1000).unwrap(),
            ),
        ))
        .insert(transform)
        .id();
    ev_spawn_vessel.send(SpawnVessel {
        entity,
        turrets: vec![Some(TurretType::Rocket)],
        health: Health::new(entity, 100.0),
    });
}

fn despawn_enemies(mut commands: Commands, q_enemies: Query<(Entity, &Health)>) {
    for (entity, health) in &q_enemies {
        if health.health <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_enemy_move_targets(
    mut q_enemies: Query<(&Transform, &ShipStats, &mut Enemy), Without<Player>>,
    q_player: Query<(&Transform, &ShipStats), (With<Player>, Without<Enemy>)>,
) {
    let (player_transform, player_ship_stats) = match q_player.get_single() {
        Ok(p) => (p.0, p.1),
        Err(_) => return,
    };

    let p = player_transform.translation.truncate();
    let p_perp = player_transform.local_y().truncate().perp()
        * (player_ship_stats.current_speed).max(MIN_PLAYER_DISTANCE);

    for (enemy_transform, _enemy_ship_stats, mut enemy) in &mut q_enemies {
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

fn steer_enemies(mut q_enemies: Query<(&Transform, &mut ShipStats, &Enemy)>) {
    for (transform, mut ship_stats, enemy) in &mut q_enemies {
        let angle = enemy
            .target_point
            .angle_between(transform.local_y().truncate());
        let steer_direction = if angle.abs() <= MIN_ANGLE_THRESHOLD {
            0.0
        } else if angle < 0.0 {
            1.0
        } else {
            -1.0
        };

        ship_stats.current_steering_direction = steer_direction;
    }
}

fn accelerate_enemies(time: Res<Time>, mut q_enemies: Query<(&Transform, &mut ShipStats, &Enemy)>) {
    for (_enemy_transform, mut enemy_ship_stats, enemy) in &mut q_enemies {
        let speed = enemy
            .target_point
            .length_squared()
            .min(MAX_DISTANCE_SQUARED)
            / MAX_DISTANCE_SQUARED;
        if enemy_ship_stats.current_speed / enemy_ship_stats.max_speed < speed {
            enemy_ship_stats.current_speed += enemy_ship_stats.delta_speed * time.delta_seconds();
        } else {
            enemy_ship_stats.current_speed -= enemy_ship_stats.delta_speed * time.delta_seconds();
        }
    }
}
