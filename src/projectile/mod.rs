pub mod rocket;
mod rocket_explosion;

use bevy::prelude::*;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            rocket::RocketPlugin,
            rocket_explosion::RocketExplosionPlugin,
        ));
    }
}
