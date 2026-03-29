mod ball;
mod court;
mod npc;
mod player;
mod ritual;
mod scoring;
mod ui;

use bevy::prelude::*;

pub struct FeaturesPlugin;

impl Plugin for FeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            court::CourtPlugin,
            player::PlayerPlugin,
            ball::BallPlugin,
            scoring::ScoringPlugin,
            ritual::RitualPlugin,
            ui::GameUiPlugin,
        ));
    }
}
