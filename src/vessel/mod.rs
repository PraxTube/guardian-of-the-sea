pub mod ship;

use bevy::prelude::*;

pub struct GuardianVesselPlugin;

impl Plugin for GuardianVesselPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ship::GuardianShipPlugin);
    }
}
