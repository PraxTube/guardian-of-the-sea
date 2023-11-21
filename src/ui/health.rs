use bevy::prelude::*;

use crate::{
    projectile::ProjectileCollision,
    vessel::{ship::move_ships, SpawnVessel},
    GameState,
};

#[derive(Component, Clone)]
pub struct Health {
    pub entity: Entity,
    pub health: f32,
    pub max_health: f32,
    pub size: f32,
}

impl Health {
    pub fn new(entity: Entity, max_health: f32, size: f32) -> Self {
        Self {
            entity,
            health: max_health,
            max_health,
            size,
        }
    }

    pub fn health_bar_offset(&self) -> Vec3 {
        Vec3::new(-30.0, -40.0, 0.0) * self.size
    }

    pub fn health_bar_scale(&self) -> Vec3 {
        Vec3::new(60.0, 7.5, 1.0) * self.size
    }
}

#[derive(Component)]
struct HealthBar {
    entity: Entity,
}

#[derive(Component)]
struct HealthBarFill;

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
                health_transform.translation + health.health_bar_offset();
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
    health: &Health,
) -> Entity {
    commands
        .spawn((
            HealthBar { entity },
            SpatialBundle {
                transform: Transform::from_translation(spawn_position + health.health_bar_offset()),
                ..default()
            },
        ))
        .id()
}

fn spawn_background(commands: &mut Commands, health: &Health) -> Entity {
    let transform = Transform::from_scale(health.health_bar_scale()).with_translation(Vec3::new(
        health.health_bar_scale().x / 2.0,
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

fn spawn_fill(commands: &mut Commands, health: &Health) -> Entity {
    let transform = Transform::from_scale(health.health_bar_scale()).with_translation(Vec3::new(
        health.health_bar_scale().x / 2.0,
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

fn spawn_health_bar(commands: &mut Commands, ev: SpawnVessel) {
    let container = spawn_container(commands, Vec3::ZERO, ev.entity, &ev.health);
    let background = spawn_background(commands, &ev.health);
    let fill_container = spawn_fill_container(commands);
    let fill = spawn_fill(commands, &ev.health);

    commands.entity(fill_container).push_children(&[fill]);
    commands
        .entity(container)
        .push_children(&[fill_container, background]);
}

fn spawn_health_bars(mut commands: Commands, mut ev_spawn_health: EventReader<SpawnVessel>) {
    for ev in ev_spawn_health.read() {
        if let Some(mut entity) = commands.get_entity(ev.entity) {
            entity.insert(ev.health.clone());
            spawn_health_bar(&mut commands, ev.clone());
        }
    }
}

fn apply_projectile_damage(
    mut q_healths: Query<&mut Health>,
    mut ev_projectile_collision: EventReader<ProjectileCollision>,
) {
    for ev in ev_projectile_collision.read() {
        if let Ok(mut health) = q_healths.get_mut(ev.target) {
            health.health -= ev.projectile.damage;
        }
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_health_bars.after(move_ships),
                fill_health_bars,
                spawn_health_bars,
                apply_projectile_damage,
            )
                .run_if(in_state(GameState::Gaming)),
        );
    }
}
