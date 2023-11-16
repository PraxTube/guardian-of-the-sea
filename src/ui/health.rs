use bevy::prelude::*;

use crate::{projectile::rocket::RocketCollision, GameState, ShipStats};

#[derive(Component, Clone)]
pub struct Health {
    pub entity: Entity,
    pub health: f32,
    pub max_health: f32,
    pub ship_stats: ShipStats,
}

impl Health {
    pub fn new(entity: Entity, max_health: f32, ship_stats: ShipStats) -> Self {
        Self {
            entity,
            health: max_health,
            max_health,
            ship_stats,
        }
    }
}

#[derive(Component)]
struct HealthBar {
    entity: Entity,
}

#[derive(Component)]
struct HealthBarFill;

#[derive(Event)]
pub struct SpawnHealth {
    pub entity: Entity,
    pub health: Health,
}

fn move_health_bars(
    mut health_bars: Query<(&HealthBar, &mut Transform), (Without<Health>, Without<HealthBarFill>)>,
    healths: Query<(&Transform, &Health), Without<HealthBar>>,
) {
    for (health_transform, health) in &healths {
        for (health_bar, mut health_bar_transform) in &mut health_bars {
            if health.entity != health_bar.entity {
                continue;
            }

            health_bar_transform.translation =
                health_transform.translation + health.ship_stats.health_bar_offset;
        }
    }
}

fn fill_health_bar(
    health_bar_fills: &mut Query<
        (&mut Transform, &HealthBarFill),
        (Without<Health>, Without<HealthBar>),
    >,
    children: &Children,
    health: &Health,
) {
    for &child in children {
        let health_bar_fill = health_bar_fills.get_mut(child);
        if let Ok(mut fill) = health_bar_fill {
            let x_fill = (health.health / health.max_health).clamp(0.0, 1.0);
            fill.0.scale = Vec3::new(x_fill, fill.0.scale.y, fill.0.scale.z);
        }
    }
}

fn fill_health_bars(
    mut health_bars: Query<
        (&HealthBar, &Children, &mut Visibility),
        (Without<Health>, Without<HealthBarFill>),
    >,
    mut health_bar_fills: Query<
        (&mut Transform, &HealthBarFill),
        (Without<Health>, Without<HealthBar>),
    >,
    healths: Query<&Health, Without<HealthBar>>,
) {
    for (health_bar, children, mut health_bar_visibility) in &mut health_bars {
        *health_bar_visibility = Visibility::Hidden;
        for health in &healths {
            if health.entity != health_bar.entity {
                continue;
            }

            *health_bar_visibility = Visibility::Visible;
            fill_health_bar(&mut health_bar_fills, children, health);
        }
    }
}

fn spawn_container(
    commands: &mut Commands,
    spawn_position: Vec3,
    entity: Entity,
    ship_stats: &ShipStats,
) -> Entity {
    commands
        .spawn((
            HealthBar { entity },
            SpatialBundle {
                transform: Transform::from_translation(
                    spawn_position + ship_stats.health_bar_offset,
                ),
                ..default()
            },
        ))
        .id()
}

fn spawn_background(commands: &mut Commands, ship_stats: &ShipStats) -> Entity {
    let transform = Transform::from_scale(ship_stats.health_bar_scale).with_translation(Vec3::new(
        ship_stats.health_bar_scale.x / 2.0,
        0.0,
        10.0,
    ));
    commands
        .spawn((SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            transform,
            ..default()
        },))
        .id()
}

fn spawn_fill_container(commands: &mut Commands) -> Entity {
    commands
        .spawn((HealthBarFill, SpatialBundle::default()))
        .id()
}

fn spawn_fill(commands: &mut Commands, ship_stats: &ShipStats) -> Entity {
    let transform = Transform::from_scale(ship_stats.health_bar_scale).with_translation(Vec3::new(
        ship_stats.health_bar_scale.x / 2.0,
        0.0,
        20.0,
    ));
    commands
        .spawn((SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.0, 0.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            transform,
            ..default()
        },))
        .id()
}

fn spawn_health_bars(mut commands: Commands, mut ev_spawn_health: EventReader<SpawnHealth>) {
    for ev in ev_spawn_health.read() {
        if let Some(mut entity) = commands.get_entity(ev.entity) {
            entity.insert(ev.health.clone());

            let container = spawn_container(
                &mut commands,
                Vec3::default(),
                ev.entity,
                &ev.health.ship_stats,
            );
            let background = spawn_background(&mut commands, &ev.health.ship_stats);
            let fill_container = spawn_fill_container(&mut commands);
            let fill = spawn_fill(&mut commands, &ev.health.ship_stats);

            commands.entity(fill_container).push_children(&[fill]);
            commands
                .entity(container)
                .push_children(&[fill_container, background]);
        }
    }
}

fn apply_rocket_damage(
    mut q_healths: Query<&mut Health>,
    mut ev_rocket_collision: EventReader<RocketCollision>,
) {
    for ev in ev_rocket_collision.read() {
        if let Ok(mut health) = q_healths.get_mut(ev.entity) {
            health.health -= ev.rocket.damage;
        }
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_health_bars,
                fill_health_bars,
                spawn_health_bars,
                apply_rocket_damage,
            )
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<SpawnHealth>();
    }
}
