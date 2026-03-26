mod constants;
mod coords;
mod errors;
mod events;
mod game_state;
mod math;
mod save;
mod settings;
mod time;

use bevy::prelude::*;

pub use constants::*;
pub use game_state::GameState;
pub use settings::SessionConfig;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SessionConfig::default());
    }
}
