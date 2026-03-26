use bevy::prelude::*;

use crate::app::startup;
use crate::core::GameState;

pub struct AppFlowPlugin;

impl Plugin for AppFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::StartScreen), startup::spawn_start_screen)
            .add_systems(
                Update,
                (
                    startup::update_start_screen_selection,
                    startup::sync_start_screen_text,
                    startup::confirm_start_screen_selection,
                )
                    .run_if(in_state(GameState::StartScreen)),
            )
            .add_systems(
                OnExit(GameState::StartScreen),
                startup::despawn_start_screen,
            )
            .add_systems(OnEnter(GameState::InGame), startup::spawn_in_game_hud)
            .add_systems(
                Update,
                (
                    startup::handle_in_game_hotkeys,
                    startup::handle_in_game_buttons,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), startup::despawn_in_game_hud)
            .add_systems(
                OnEnter(GameState::Restarting),
                startup::advance_restart_state,
            );
    }
}
