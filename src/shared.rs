mod assets;
mod audio;
mod camera;
mod debug_draw;
mod input;
mod physics;
mod random;
mod ui;

use bevy::prelude::*;
pub use camera::MainCamera;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(camera::CameraPlugin);
    }
}
