pub mod camera;

pub use camera::MainCamera;

use bevy::prelude::*;

pub struct GuardianWorldPlugin;

impl Plugin for GuardianWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::GuardianCameraPlugin,));
    }
}
