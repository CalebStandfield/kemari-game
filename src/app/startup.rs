use bevy::prelude::*;
use bevy::ui::{percent, px};

use crate::core::{GameState, SessionConfig};

#[derive(Component)]
pub struct StartScreenRoot;

#[derive(Component)]
pub struct StartScreenSelectionText;

#[derive(Resource, Debug, Clone, Copy)]
pub struct StartScreenSelection {
    pub player_count: usize,
}

pub fn spawn_start_screen(mut commands: Commands) {
    commands.insert_resource(StartScreenSelection { player_count: 1 });

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.92)),
            StartScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("KEMARI"),
                TextFont::from_font_size(56.0),
                TextColor(Color::srgb(0.95, 0.92, 0.80)),
            ));
            parent.spawn((
                Text::new("Select number of players: 1"),
                TextFont::from_font_size(28.0),
                TextColor(Color::srgb(0.90, 0.90, 0.90)),
                StartScreenSelectionText,
            ));
            parent.spawn((
                Text::new("Press 1-8 to choose, then Enter to start"),
                TextFont::from_font_size(22.0),
                TextColor(Color::srgb(0.72, 0.78, 0.82)),
            ));
        });
}

pub fn update_start_screen_selection(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<StartScreenSelection>,
) {
    if let Some(player_count) = read_selected_player_count(&keyboard_input) {
        selection.player_count = player_count;
    }
}

pub fn sync_start_screen_text(
    selection: Res<StartScreenSelection>,
    mut text_query: Query<&mut Text, With<StartScreenSelectionText>>,
) {
    if !selection.is_changed() {
        return;
    }

    for mut text in &mut text_query {
        **text = format!("Select number of players: {}", selection.player_count);
    }
}

pub fn confirm_start_screen_selection(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selection: Res<StartScreenSelection>,
    mut session_config: ResMut<SessionConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter)
        || keyboard_input.just_pressed(KeyCode::NumpadEnter)
        || keyboard_input.just_pressed(KeyCode::Space)
    {
        session_config.player_count = selection.player_count;
        next_state.set(GameState::InGame);
    }
}

pub fn despawn_start_screen(
    mut commands: Commands,
    root_query: Query<Entity, With<StartScreenRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<StartScreenSelection>();
}

fn read_selected_player_count(keyboard_input: &ButtonInput<KeyCode>) -> Option<usize> {
    if keyboard_input.just_pressed(KeyCode::Digit1) || keyboard_input.just_pressed(KeyCode::Numpad1)
    {
        return Some(1);
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) || keyboard_input.just_pressed(KeyCode::Numpad2)
    {
        return Some(2);
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) || keyboard_input.just_pressed(KeyCode::Numpad3)
    {
        return Some(3);
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) || keyboard_input.just_pressed(KeyCode::Numpad4)
    {
        return Some(4);
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) || keyboard_input.just_pressed(KeyCode::Numpad5)
    {
        return Some(5);
    }
    if keyboard_input.just_pressed(KeyCode::Digit6) || keyboard_input.just_pressed(KeyCode::Numpad6)
    {
        return Some(6);
    }
    if keyboard_input.just_pressed(KeyCode::Digit7) || keyboard_input.just_pressed(KeyCode::Numpad7)
    {
        return Some(7);
    }
    if keyboard_input.just_pressed(KeyCode::Digit8) || keyboard_input.just_pressed(KeyCode::Numpad8)
    {
        return Some(8);
    }

    None
}
