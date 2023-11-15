use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{GameAssets, GameState};

pub struct GuardianEnemyPlugin;

impl Plugin for GuardianEnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gaming), spawn_dummy_enemy);
    }
}

#[derive(Component)]
pub struct Enemy;

pub fn spawn_dummy_enemy(mut commands: Commands, assets: Res<GameAssets>) {
    let transform = Transform::from_translation(Vec3::new(500.0, 500.0, 0.0));
    let collider = Collider::capsule(Vec2::new(0.0, -20.0), Vec2::new(0.0, 20.0), 15.0);

    commands.spawn((
        Enemy,
        collider,
        CollisionGroups::new(
            Group::from_bits(0b0100).unwrap(),
            Group::from_bits(0b1000).unwrap(),
        ),
        SpriteBundle {
            transform,
            texture: assets.dummy_enemy.clone(),
            ..default()
        },
    ));
}
