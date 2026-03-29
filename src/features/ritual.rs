mod fail_states;
mod patterns;
mod rhythm;
mod rules;
mod systems;

use bevy::prelude::*;

pub use rhythm::EleganceMeter;

pub struct RitualPlugin;

impl Plugin for RitualPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<rhythm::EleganceMeter>()
            .init_resource::<patterns::TouchPatternState>()
            .add_systems(
                OnExit(crate::core::GameState::InGame),
                systems::reset_match_ritual_state,
            )
            .add_systems(
                Update,
                systems::update_elegance
                    .in_set(crate::core::GameplaySet::Ritual)
                    .run_if(in_state(crate::core::GameState::InGame)),
            );
    }
}
