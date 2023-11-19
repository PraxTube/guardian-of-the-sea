pub mod ship;

use bevy::prelude::*;

use crate::{turret::TurretType, ui::health::Health};

#[derive(Event, Clone)]
pub struct SpawnVessel {
    pub entity: Entity,
    pub stats_scale: f32,
    pub turrets: Vec<Option<TurretType>>,
    pub health: Health,
}

pub struct GuardianVesselPlugin;

impl Plugin for GuardianVesselPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ship::GuardianShipPlugin)
            .add_event::<SpawnVessel>();
    }
}
