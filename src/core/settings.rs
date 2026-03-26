use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct SessionConfig {
    pub player_count: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self { player_count: 1 }
    }
}
