use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, CollisionGroups, Group};

use crate::{turret::TurretStats, GameAssets, GameState};

pub struct GuardianShipPlugin;

impl Plugin for GuardianShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_ships, steer_ships).run_if(in_state(GameState::Gaming)),
        );
    }
}

#[derive(Component, Clone, Default)]
pub struct ShipStats {
    pub acceleration: Vec2,
    pub delta_steering: f32,
    pub delta_speed: f32,
    pub current_speed: f32,
    pub current_steering_direction: f32,
    pub drag: f32,
    pub traction: f32,

    pub min_speed: f32,
    pub max_speed: f32,

    pub health_bar_size: f32,

    pub dash: bool,
}

impl ShipStats {
    pub fn health_bar_offset(&self) -> Vec3 {
        Vec3::new(-30.0, -40.0, 0.0) * self.health_bar_size
    }

    pub fn health_bar_scale(&self) -> Vec3 {
        Vec3::new(60.0, 7.5, 1.0) * self.health_bar_size
    }
}

#[derive(Bundle)]
pub struct ShipVessel {
    collider: Collider,
    collision_groups: CollisionGroups,
    ship_stats: ShipStats,
    turret_stats: TurretStats,
    sprite: SpriteBundle,
}

#[derive(Bundle)]
pub struct SmallShip1 {
    ship_vessel: ShipVessel,
}

impl SmallShip1 {
    pub fn new(assets: &Res<GameAssets>, collision_layer: u32, collision_mask: u32) -> Self {
        Self {
            ship_vessel: ShipVessel {
                collider: Collider::capsule(Vec2::new(0.0, -20.0), Vec2::new(0.0, 20.0), 15.0),
                collision_groups: CollisionGroups::new(
                    Group::from_bits(collision_layer).unwrap(),
                    Group::from_bits(collision_mask).unwrap(),
                ),
                ship_stats: ShipStats {
                    delta_steering: 4.0,
                    delta_speed: 200.0,
                    min_speed: -150.0,
                    max_speed: 500.0,
                    health_bar_size: 1.0,
                    ..default()
                },
                turret_stats: TurretStats {
                    turret_offsets: vec![Vec2::ZERO],
                },
                sprite: SpriteBundle {
                    texture: assets.small_ship_1.clone(),
                    ..default()
                },
            },
        }
    }
}

#[derive(Bundle)]
pub struct BigShip {
    ship_vessel: ShipVessel,
}

impl BigShip {
    pub fn new(assets: &Res<GameAssets>, collision_layer: u32, collision_mask: u32) -> Self {
        Self {
            ship_vessel: ShipVessel {
                collider: Collider::capsule(Vec2::new(0.0, -90.0), Vec2::new(0.0, 90.0), 40.0),
                collision_groups: CollisionGroups::new(
                    Group::from_bits(collision_layer).unwrap(),
                    Group::from_bits(collision_mask).unwrap(),
                ),
                ship_stats: ShipStats {
                    delta_steering: 1.5,
                    delta_speed: 1000.0,
                    drag: 1.0,
                    min_speed: -100.0,
                    max_speed: 1000.0,
                    health_bar_size: 4.0,
                    ..default()
                },
                turret_stats: TurretStats {
                    turret_offsets: vec![
                        Vec2::new(-16.0, -16.0),
                        Vec2::new(16.0, -16.0),
                        Vec2::new(-16.0, 16.0),
                        Vec2::new(16.0, 16.0),
                        Vec2::new(-16.0, 48.0),
                        Vec2::new(16.0, 48.0),
                    ],
                },
                sprite: SpriteBundle {
                    texture: assets.boat.clone(),
                    ..default()
                },
            },
        }
    }
}

pub fn move_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &mut ShipStats)>) {
    for (mut transform, mut ship_stats) in &mut ships {
        if ship_stats.dash {
            let dir = transform.local_y();
            transform.translation += dir * ship_stats.max_speed * 5.0 * time.delta_seconds();
            ship_stats.acceleration = dir.truncate() * ship_stats.max_speed;
            continue;
        }

        transform.translation += ship_stats.acceleration.extend(0.0) * time.delta_seconds();

        let drag = 1.0 - ship_stats.drag * time.delta_seconds();
        ship_stats.acceleration *= drag;
        ship_stats.acceleration = ship_stats
            .acceleration
            .clamp_length(0.0, ship_stats.max_speed);

        let speed = ship_stats.acceleration.length();
        if speed == 0.0 {
            return;
        }

        let traction = ship_stats.traction * time.delta_seconds();
        ship_stats.acceleration = ship_stats
            .acceleration
            .normalize()
            .lerp(transform.local_y().truncate(), traction)
            * speed;
    }
}

pub fn steer_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &ShipStats)>) {
    for (mut transform, ship_stats) in &mut ships {
        let limit = (ship_stats.max_speed / 2.0).powi(2);
        let speed_factor = ship_stats.acceleration.length_squared().min(limit) / limit;
        let drifting_factor = if ship_stats.traction == 0.0 { 2.0 } else { 1.0 };
        let rotation = ship_stats.current_steering_direction
            * ship_stats.delta_steering
            * speed_factor
            * drifting_factor
            * time.delta_seconds();
        transform.rotate_z(rotation);
    }
}
