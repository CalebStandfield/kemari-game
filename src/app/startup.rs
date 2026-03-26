use bevy::prelude::*;
use bevy::ui::{percent, px};

use crate::core::{GameState, SessionConfig};

#[derive(Component)]
pub struct StartScreenRoot;

#[derive(Component)]
pub struct StartScreenSelectionText;

#[derive(Component)]
pub struct InGameHudRoot;

#[derive(Component)]
pub struct BackToStartButton;

#[derive(Component)]
pub struct ResetGameButton;

#[derive(Resource, Debug, Clone, Copy)]
pub struct StartScreenSelection {
    pub player_count: usize,
}

pub fn spawn_start_screen(mut commands: Commands, session_config: Res<SessionConfig>) {
    let initial_count = session_config.player_count.clamp(1, 8);
    commands.insert_resource(StartScreenSelection {
        player_count: initial_count,
    });

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
                Text::new(format!("Select number of players: {initial_count}")),
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

pub fn spawn_in_game_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                top: px(16),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.72)),
            InGameHudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Match Controls"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.95, 0.92, 0.80)),
            ));
            parent.spawn((
                Text::new("Esc = Back to Start, R = Reset Match"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.74, 0.80, 0.84)),
            ));

            parent
                .spawn((Node {
                    display: Display::Flex,
                    column_gap: px(8),
                    ..default()
                },))
                .with_children(|buttons| {
                    buttons
                        .spawn((
                            Button,
                            Node {
                                width: px(150),
                                height: px(34),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.19, 0.22)),
                            BackToStartButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Back to Start"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(0.93, 0.94, 0.95)),
                            ));
                        });

                    buttons
                        .spawn((
                            Button,
                            Node {
                                width: px(130),
                                height: px(34),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.19, 0.22)),
                            ResetGameButton,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Reset Match"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(0.93, 0.94, 0.95)),
                            ));
                        });
                });
        });
}

pub fn despawn_in_game_hud(mut commands: Commands, root_query: Query<Entity, With<InGameHudRoot>>) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_in_game_hotkeys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::StartScreen);
        return;
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Restarting);
    }
}

pub fn handle_in_game_buttons(
    mut button_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&BackToStartButton>,
            Option<&ResetGameButton>,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<BackToStartButton>, With<ResetGameButton>)>,
        ),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut background_color, back_button, reset_button) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb(0.30, 0.34, 0.38));

                if back_button.is_some() {
                    next_state.set(GameState::StartScreen);
                    continue;
                }

                if reset_button.is_some() {
                    next_state.set(GameState::Restarting);
                }
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb(0.22, 0.26, 0.30));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb(0.16, 0.19, 0.22));
            }
        }
    }
}

pub fn advance_restart_state(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
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
