use bevy::prelude::*;

use crate::{GameAssets, GameState};

use super::{ProjectileDespawn, ProjectileType};

const GFX_SCALE: Vec3 = Vec3::splat(4.0);

#[derive(Component, Default)]
struct RocketExplosion {
    disabled: bool,
}

#[derive(Component)]
struct ExplosionAnimationTimer {
    timer: Timer,
}

impl Default for ExplosionAnimationTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.075, TimerMode::Repeating),
        }
    }
}

fn spawn_rocket_explosion(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_projectile_despawn: EventReader<ProjectileDespawn>,
) {
    for ev in ev_projectile_despawn.read() {
        if ev.projectile.projectile_type != ProjectileType::Rocket {
            continue;
        }

        let transform = Transform::from_translation(ev.position).with_scale(GFX_SCALE);
        commands.spawn((
            RocketExplosion::default(),
            ExplosionAnimationTimer::default(),
            SpriteSheetBundle {
                transform,
                texture_atlas: assets.explosion.clone(),
                ..default()
            },
        ));
    }
}

fn animate_rocket_explosions(
    time: Res<Time>,
    mut query: Query<(
        &mut RocketExplosion,
        &mut ExplosionAnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut rocket_explosion, mut timer, mut sprite) in &mut query {
        if sprite.index > 7 {
            rocket_explosion.disabled = true;
            continue;
        }

        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            if sprite.index == 7 {
                rocket_explosion.disabled = true;
            } else {
                sprite.index += 1;
            }
        }
    }
}

fn despawn_rocket_explosions(mut commands: Commands, query: Query<(Entity, &RocketExplosion)>) {
    for (entity, rocket_explosion) in &query {
        if rocket_explosion.disabled {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct RocketExplosionPlugin;

impl Plugin for RocketExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_rocket_explosion,
                animate_rocket_explosions,
                despawn_rocket_explosions,
            )
                .run_if(in_state(GameState::Gaming)),
        );
    }
}
