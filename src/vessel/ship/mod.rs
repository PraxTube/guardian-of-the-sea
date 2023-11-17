use bevy::prelude::*;
use bevy_rapier2d::prelude::Collider;

use crate::{GameAssets, GameState};

pub struct GuardianShipPlugin;

impl Plugin for GuardianShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_ships, steer_ships).run_if(in_state(GameState::Gaming)),
        );
    }
}

#[derive(Component, Clone)]
pub struct ShipStats {
    pub delta_steering: f32,
    pub delta_speed: f32,
    pub current_speed: f32,
    pub current_steering_direction: f32,

    pub min_speed: f32,
    pub max_speed: f32,

    pub health_bar_offset: Vec3,
    pub health_bar_scale: Vec3,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            delta_steering: 4.0,
            delta_speed: 100.0,
            current_speed: 0.0,
            current_steering_direction: 0.0,
            min_speed: -150.0,
            max_speed: 500.0,
            health_bar_offset: Vec3::new(-30.0, -40.0, 0.0),
            health_bar_scale: Vec3::new(60.0, 7.5, 1.0),
        }
    }
}

#[derive(Bundle)]
pub struct SmallShip1 {
    collider: Collider,
    ship_stats: ShipStats,
    sprite: SpriteBundle,
}

impl SmallShip1 {
    pub fn new(assets: &Res<GameAssets>) -> Self {
        Self {
            collider: Collider::capsule(Vec2::new(0.0, -20.0), Vec2::new(0.0, 20.0), 15.0),
            ship_stats: ShipStats::default(),
            sprite: SpriteBundle {
                texture: assets.small_ship_1.clone(),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct BigShip {
    collider: Collider,
    ship_stats: ShipStats,
    sprite: SpriteBundle,
}

impl BigShip {
    pub fn new(assets: &Res<GameAssets>) -> Self {
        Self {
            collider: Collider::capsule(Vec2::new(0.0, -90.0), Vec2::new(0.0, 90.0), 40.0),
            ship_stats: ShipStats {
                delta_steering: 2.0,
                delta_speed: 75.0,
                min_speed: -100.0,
                max_speed: 400.0,
                health_bar_offset: Vec3::new(-30.0, -40.0, 0.0) * 4.0,
                health_bar_scale: Vec3::new(60.0, 7.5, 1.0) * 4.0,
                ..default()
            },
            sprite: SpriteBundle {
                texture: assets.boat.clone(),
                ..default()
            },
        }
    }
}

pub fn move_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &ShipStats)>) {
    for (mut transform, ship_stats) in &mut ships {
        let direction = transform.local_y();
        transform.translation += direction * ship_stats.current_speed * time.delta_seconds();
    }
}

pub fn steer_ships(time: Res<Time>, mut ships: Query<(&mut Transform, &ShipStats)>) {
    for (mut transform, ship_stats) in &mut ships {
        let rotation = ship_stats.delta_steering
            * ship_stats.current_steering_direction
            * time.delta_seconds();
        transform.rotate_z(rotation);
    }
}
