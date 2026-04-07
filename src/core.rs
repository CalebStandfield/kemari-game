mod constants;
mod coords;
mod errors;
mod events;
mod game_state;
mod math;
mod save;
mod settings;
mod tags;
mod time;

use bevy::prelude::*;

pub use constants::*;
pub use events::*;
pub use game_state::GameState;
pub use settings::SessionConfig;
pub use tags::PlayerBody;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SessionConfig::default())
            .add_message::<PlayerTouchAttemptEvent>()
            .add_message::<BallTouchedEvent>()
            .add_message::<BallWhiffedEvent>()
            .add_message::<BallHitGroundEvent>()
            .add_message::<PassResolutionEvent>()
            .configure_sets(
                Update,
                (
                    GameplaySet::PlayerInput,
                    GameplaySet::BallResolve,
                    GameplaySet::Scoring,
                    GameplaySet::Ritual,
                    GameplaySet::Ui,
                )
                    .chain(),
            );
    }
}
