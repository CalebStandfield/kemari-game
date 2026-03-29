mod chain;
mod elegance;
mod events;
mod rules;
mod systems;

use bevy::prelude::*;

pub use chain::ChainCounter;

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChainCounter>().add_systems(
            Update,
            systems::update_chain
                .in_set(crate::core::GameplaySet::Scoring)
                .run_if(in_state(crate::core::GameState::InGame)),
        );
    }
}
