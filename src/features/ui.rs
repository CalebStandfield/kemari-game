mod debug;
mod hud;
mod menu;
mod pause;
mod systems;

use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::core::GameState::InGame),
            hud::spawn_gameplay_hud,
        )
        .add_systems(
            OnExit(crate::core::GameState::InGame),
            hud::despawn_gameplay_hud,
        )
        .add_systems(
            Update,
            systems::update_gameplay_hud
                .in_set(crate::core::GameplaySet::Ui)
                .run_if(in_state(crate::core::GameState::InGame)),
        );
    }
}
