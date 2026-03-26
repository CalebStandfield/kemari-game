use bevy::prelude::*;

#[derive(States, Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    StartScreen,
    InGame,
}
