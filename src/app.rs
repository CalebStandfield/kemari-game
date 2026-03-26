mod config;
mod plugins;
mod startup;
mod states;

use bevy::prelude::*;
use bevy::window::WindowPlugin;

use crate::app::plugins::AppFlowPlugin;
use crate::core::CorePlugin;
use crate::features::FeaturesPlugin;
use crate::shared::SharedPlugin;

pub struct KemariAppPlugin;

impl Plugin for KemariAppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(config::clear_color()))
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(config::primary_window()),
                ..default()
            }))
            .add_plugins((CorePlugin, AppFlowPlugin, SharedPlugin, FeaturesPlugin));
    }
}
