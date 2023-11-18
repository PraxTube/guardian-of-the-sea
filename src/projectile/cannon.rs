use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    collision::PROJECTILE_LAYER,
    turret::{TurretTriggered, TurretType},
    utils::anim_sprite::{AnimSprite, AnimSpriteTimer},
    GameAssets, GameState,
};

use super::{Projectile, ProjectileTimer, ProjectileType};

const DAMAGE: f32 = 1.0;
const LIFE_TIME: f32 = 2.0;
const CANNON_SIZE: f32 = 2.0;

#[derive(Component, Clone)]
pub struct Cannon {
    source_velocity: Vec2,
    current_speed: f32,
}

impl Default for Cannon {
    fn default() -> Self {
        Self {
            source_velocity: Vec2::default(),
            current_speed: 1200.0,
        }
    }
}

impl Cannon {
    pub fn new(source_velocity: Vec2) -> Self {
        Self {
            source_velocity,
            ..default()
        }
    }
}

fn spawn_cannon(commands: &mut Commands, assets: &Res<GameAssets>, ev: &TurretTriggered) {
    let transform = Transform::from_translation(ev.source_transform.translation)
        .with_rotation(ev.source_transform.rotation)
        .with_scale(Vec3::splat(CANNON_SIZE));
    let collider = Collider::capsule(Vec2::default(), Vec2::new(0.0, 6.0), 3.0);
    let collision_groups = CollisionGroups::new(
        Group::from_bits(PROJECTILE_LAYER).unwrap(),
        Group::from_bits(ev.turret_mask).unwrap(),
    );

    commands.spawn((
        Cannon::new(ev.source_velocity),
        Projectile::new(ProjectileType::Cannon, ev.source, ev.turret_mask, DAMAGE),
        ProjectileTimer::new(LIFE_TIME),
        AnimSprite::new(4, true),
        AnimSpriteTimer::default(),
        SpriteSheetBundle {
            transform,
            texture_atlas: assets.cannon.clone(),
            ..default()
        },
        collider,
        collision_groups,
    ));
}

fn spawn_cannons(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_turret_triggered: EventReader<TurretTriggered>,
) {
    for ev in ev_turret_triggered.read() {
        if ev.turret_type == TurretType::Cannon {
            spawn_cannon(&mut commands, &assets, ev);
        }
    }
}

fn move_cannons(time: Res<Time>, mut q_cannons: Query<(&mut Transform, &Cannon)>) {
    for (mut transform, cannon) in &mut q_cannons {
        let direction = transform.local_y();
        transform.translation += (direction * cannon.current_speed
            + cannon.source_velocity.extend(0.0))
            * time.delta_seconds();
    }
}

pub struct CannonPlugin;

impl Plugin for CannonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_cannons, move_cannons)
                .chain()
                .run_if(in_state(GameState::Gaming)),
        );
    }
}
