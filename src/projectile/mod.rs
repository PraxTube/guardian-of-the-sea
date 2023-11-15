mod rocket;

use bevy::prelude::*;

use crate::GameState;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                rocket::shoot_rockets,
                rocket::fire_rockets,
                rocket::move_rockets,
                rocket::despawn_rockets,
                rocket::check_rocket_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Gaming)),
        )
        .add_event::<rocket::RocketCollision>()
        .add_event::<rocket::RocketFired>();
    }
}
