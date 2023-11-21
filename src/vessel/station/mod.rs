use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, CollisionGroups, Group};

use crate::{turret::TurretStats, GameAssets, GameState};

pub struct GuardianStationPlugin;

impl Plugin for GuardianStationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (rotate_stations).run_if(in_state(GameState::Gaming)),
        );
    }
}

#[derive(Component, Clone, Default)]
pub struct StationStats {
    pub delta_steering: f32,
    pub current_steering_direction: f32,
}

impl StationStats {}

#[derive(Bundle)]
pub struct StationVessel {
    collider: Collider,
    collision_groups: CollisionGroups,
    station_stats: StationStats,
    turret_stats: TurretStats,
    sprite: SpriteBundle,
}

#[derive(Bundle)]
pub struct SmallStation1 {
    station_vessel: StationVessel,
}

impl SmallStation1 {
    pub fn new(assets: &Res<GameAssets>, collision_layer: u32, collision_mask: u32) -> Self {
        Self {
            station_vessel: StationVessel {
                collider: Collider::cuboid(48.0, 48.0),
                collision_groups: CollisionGroups::new(
                    Group::from_bits(collision_layer).unwrap(),
                    Group::from_bits(collision_mask).unwrap(),
                ),
                station_stats: StationStats {
                    delta_steering: 4.0,
                    ..default()
                },
                turret_stats: TurretStats {
                    turret_offsets: vec![Vec2::ZERO],
                },
                sprite: SpriteBundle {
                    texture: assets.station.clone(),
                    ..default()
                },
            },
        }
    }
}

pub fn rotate_stations(time: Res<Time>, mut q_stations: Query<(&mut Transform, &StationStats)>) {
    for (mut transform, station_stats) in &mut q_stations {
        let rotation = station_stats.current_steering_direction
            * station_stats.delta_steering
            * time.delta_seconds();
        transform.rotate_z(rotation);
    }
}
