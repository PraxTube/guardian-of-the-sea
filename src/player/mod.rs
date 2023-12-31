pub mod input;

use bevy::prelude::*;

use crate::collision::{PLAYER_LAYER, PROJECTILE_LAYER};
use crate::turret::TurretType;
use crate::ui::health::Health;
use crate::vessel::ship::BigShip;
use crate::vessel::SpawnVessel;
use crate::{GameAssets, GameState, ShipStats};

pub struct GuardianPlayerPlugin;

impl Plugin for GuardianPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gaming), (spawn_player_big,))
            .add_plugins((input::GuardianInputPlugin,))
            .add_systems(
                Update,
                (steer_player, accelerate_player, toggle_drift, toggle_dash)
                    .run_if(in_state(GameState::Gaming)),
            );
    }
}

#[derive(Component, Default)]
pub struct Player {}

fn spawn_player_big(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_vessel: EventWriter<SpawnVessel>,
) {
    let entity = commands
        .spawn((
            Player::default(),
            BigShip::new(&assets, PLAYER_LAYER, PROJECTILE_LAYER),
        ))
        .id();
    ev_spawn_vessel.send(SpawnVessel {
        entity,
        stats_scale: 1.0,
        turrets: vec![
            Some(TurretType::Rocket),
            Some(TurretType::Rocket),
            Some(TurretType::Rocket),
            Some(TurretType::Cannon),
            Some(TurretType::Cannon),
            Some(TurretType::Cannon),
        ],
        health: Health::new(entity, 1000.0, 4.0),
    });
}

fn steer_player(keys: Res<Input<KeyCode>>, mut q_player: Query<&mut ShipStats, With<Player>>) {
    let mut ship_stats = match q_player.get_single_mut() {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut steer_direction = 0.0;
    if keys.pressed(KeyCode::A) {
        steer_direction += 1.0;
    }
    if keys.pressed(KeyCode::D) {
        steer_direction -= 1.0;
    }
    ship_stats.current_steering_direction = steer_direction;
}

fn accelerate_player(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut q_player: Query<(&Transform, &mut ShipStats), With<Player>>,
) {
    let (transform, mut ship_stats) = match q_player.get_single_mut() {
        Ok(p) => (p.0, p.1),
        Err(_) => return,
    };

    let mut acceleration: f32 = 0.0;
    if keys.pressed(KeyCode::W) {
        acceleration += 1.0;
    }
    if keys.pressed(KeyCode::S) {
        acceleration -= 1.0;
    }

    ship_stats.drag = (-acceleration.abs() + 1.0) * 2.0;
    let speed = ship_stats.delta_speed;
    ship_stats.acceleration +=
        transform.local_y().truncate() * speed * acceleration * time.delta_seconds();
}

fn toggle_drift(keys: Res<Input<KeyCode>>, mut q_player: Query<&mut ShipStats, With<Player>>) {
    let mut ship_stats = match q_player.get_single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    if keys.pressed(KeyCode::ShiftLeft) {
        ship_stats.traction = 0.0;
    } else {
        ship_stats.traction = 5.0;
    }
}

fn toggle_dash(keys: Res<Input<KeyCode>>, mut q_player: Query<&mut ShipStats, With<Player>>) {
    let mut ship_stats = match q_player.get_single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    ship_stats.dash = keys.pressed(KeyCode::Space);
}
