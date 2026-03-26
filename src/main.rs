mod app;
mod core;
mod dev;
mod features;
mod shared;
mod tests;

use app::KemariAppPlugin;
use bevy::prelude::*;

fn main() {
    App::new().add_plugins(KemariAppPlugin).run();
}
