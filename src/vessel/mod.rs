pub mod ship;

use bevy::prelude::*;

use crate::{turret::Turret, ui::health::Health};

#[derive(Event, Clone)]
pub struct SpawnVessel {
    pub entity: Entity,
    pub turrets: Vec<Turret>,
    pub health: Health,
}

pub struct GuardianVesselPlugin;

impl Plugin for GuardianVesselPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ship::GuardianShipPlugin)
            .add_event::<SpawnVessel>();
    }
}
