pub mod input;

use bevy::prelude::*;
use bevy_rapier2d::prelude::Collider;

use crate::turret::{SpawnTurretsEvent, Turret};
use crate::ui::health::{Health, SpawnHealth};
use crate::{GameAssets, GameState, ShipStats};

pub struct GuardianPlayerPlugin;

impl Plugin for GuardianPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Gaming),
            (
                // spawn_player_small,
                spawn_player_big,
            ),
        )
        .add_plugins((input::GuardianInputPlugin,))
        .add_systems(
            Update,
            (
                steer_player,
                accelerate_player,
                toggle_player_active_momentum,
                reduce_player_speed,
            )
                .run_if(in_state(GameState::Gaming)),
        );
    }
}

#[derive(Component)]
pub struct Player {
    active_momentum: bool,
}

fn spawn_player_small(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventWriter<SpawnTurretsEvent>,
    mut ev_spawn_health: EventWriter<SpawnHealth>,
) {
    let ship_stats = ShipStats::default();
    let entity = commands
        .spawn((
            Player {
                active_momentum: true,
            },
            ship_stats.clone(),
            SpriteBundle {
                texture: assets.ship.clone(),
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
    });
}

fn spawn_player_big(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_spawn_turrets: EventWriter<SpawnTurretsEvent>,
    mut ev_spawn_health: EventWriter<SpawnHealth>,
) {
    let mut ship_stats = ShipStats::default();
    ship_stats.delta_speed *= 0.75;
    ship_stats.delta_steering *= 0.5;
    ship_stats.health_bar_offset *= 4.0;
    ship_stats.health_bar_scale *= 4.0;
    let collider = Collider::capsule(Vec2::new(0.0, -100.0), Vec2::new(0.0, 100.0), 40.0);

    let entity = commands
        .spawn((
            Player {
                active_momentum: true,
            },
            collider,
            ship_stats.clone(),
            SpriteBundle {
                texture: assets.boat.clone(),
                ..default()
            },
        ))
        .id();
    ev_spawn_turrets.send(SpawnTurretsEvent {
        turrets: vec![
            Turret::new(entity, Vec2::new(16.0, 16.0)),
            Turret::new(entity, Vec2::new(-16.0, 16.0)),
            Turret::new(entity, Vec2::new(16.0, -16.0)),
            Turret::new(entity, Vec2::new(-16.0, -16.0)),
            Turret::new(entity, Vec2::new(-16.0, 48.0)),
            Turret::new(entity, Vec2::new(16.0, 48.0)),
        ],
    });
    ev_spawn_health.send(SpawnHealth {
        entity,
        health: Health::new(entity, 10.0, ship_stats),
    });
}

fn steer_player(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(&mut Transform, &ShipStats)>,
) {
    let (mut transform, ship_stats) = player.single_mut();

    let mut steer_direction = 0.0;
    if keys.pressed(KeyCode::A) {
        steer_direction += 1.0;
    }
    if keys.pressed(KeyCode::D) {
        steer_direction -= 1.0;
    }

    if steer_direction == 0.0 {
        return;
    }

    let rotation = ship_stats.delta_steering * steer_direction * time.delta_seconds();
    transform.rotate_z(rotation);
}

fn accelerate_player(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut ShipStats>,
) {
    let mut ship_stats = player.single_mut();

    let mut acceleration = 0.0;
    if keys.pressed(KeyCode::W) {
        acceleration += 1.0;
    }
    if keys.pressed(KeyCode::S) {
        acceleration -= 1.0;
    }

    ship_stats.current_speed = (ship_stats.current_speed
        + acceleration * ship_stats.delta_speed * time.delta_seconds())
    .clamp(ship_stats.min_speed, ship_stats.max_speed);
}

fn toggle_player_active_momentum(keys: Res<Input<KeyCode>>, mut q_player: Query<&mut Player>) {
    if !keys.just_pressed(KeyCode::T) {
        return;
    }

    let mut player = q_player.single_mut();
    player.active_momentum = !player.active_momentum;
}

fn reduce_player_speed(time: Res<Time>, mut q_player: Query<(&mut ShipStats, &Player)>) {
    let (mut ship_stats, player) = q_player.single_mut();
    if player.active_momentum || ship_stats.current_speed == 0.0 {
        return;
    }

    let reduction = ship_stats.delta_speed / 2.0 * time.delta_seconds();
    if ship_stats.current_speed > 0.0 {
        ship_stats.current_speed = (ship_stats.current_speed - reduction).max(0.0);
    } else {
        ship_stats.current_speed = (ship_stats.current_speed + reduction).min(0.0);
    }
}
