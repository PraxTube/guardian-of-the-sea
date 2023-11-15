pub mod health;

use bevy::prelude::*;

pub struct GuardianUiPlugin;

impl Plugin for GuardianUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(health::HealthPlugin);
    }
}
