use bevy::prelude::*;

use crate::{
    utils::anim_sprite::{AnimSprite, AnimSpriteTimer},
    GameAssets, GameState,
};

use super::{ProjectileDespawn, ProjectileType};

const EXPLOSION_SIZE: f32 = 4.0;

fn spawn_rocket_explosion(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_projectile_despawn: EventReader<ProjectileDespawn>,
) {
    for ev in ev_projectile_despawn.read() {
        if ev.projectile.projectile_type != ProjectileType::Rocket {
            continue;
        }

        let transform =
            Transform::from_translation(ev.position).with_scale(Vec3::splat(EXPLOSION_SIZE));
        commands.spawn((
            AnimSprite::new(8, false),
            AnimSpriteTimer::default(),
            SpriteSheetBundle {
                transform,
                texture_atlas: assets.explosion.clone(),
                ..default()
            },
        ));
    }
}

pub struct RocketExplosionPlugin;

impl Plugin for RocketExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_rocket_explosion,).run_if(in_state(GameState::Gaming)),
        );
    }
}
