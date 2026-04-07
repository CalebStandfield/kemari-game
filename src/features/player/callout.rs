use bevy::prelude::*;
use bevy::ui::Val;

use super::components::{Player, PlayerCallForBall, PlayerDisplayName};

#[derive(Component)]
pub struct PlayerCalloutRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerCalloutAnchor {
    pub player: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerCalloutParts {
    pub name_text: Entity,
    pub prompt_text: Entity,
}

#[derive(Component)]
pub struct PlayerCalloutNameText;

#[derive(Component)]
pub struct PlayerCalloutPromptText;

pub fn spawn_player_callout(commands: &mut Commands, player: Entity, display_name: &str) {
    let root = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Px(crate::core::PLAYER_CALLOUT_SCREEN_WIDTH),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(
            0.03,
            0.05,
            0.08,
            crate::core::PLAYER_CALLOUT_NORMAL_BG_ALPHA,
        )))
        .insert(GlobalZIndex(20))
        .insert(Visibility::Hidden)
        .insert(PlayerCalloutRoot)
        .insert(PlayerCalloutAnchor { player })
        .id();

    let name_text = commands
        .spawn((
            Text::new(display_name),
            TextFont::from_font_size(crate::core::PLAYER_CALLOUT_NAME_FONT_SIZE),
            TextColor(color_from_tuple(
                crate::core::PLAYER_CALLOUT_NORMAL_NAME_COLOR,
            )),
            PlayerCalloutNameText,
        ))
        .id();

    let prompt_text = commands
        .spawn((
            Text::new("Pass it here"),
            TextFont::from_font_size(crate::core::PLAYER_CALLOUT_PROMPT_FONT_SIZE),
            TextColor(color_from_tuple(crate::core::PLAYER_CALLOUT_PROMPT_COLOR)),
            Visibility::Hidden,
            PlayerCalloutPromptText,
        ))
        .id();

    commands
        .entity(root)
        .add_children(&[name_text, prompt_text]);
    commands.entity(root).insert(PlayerCalloutParts {
        name_text,
        prompt_text,
    });
}

pub fn update_player_callout_positions(
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::shared::MainCamera>>,
    player_query: Query<&Transform, With<Player>>,
    mut callout_query: Query<
        (&PlayerCalloutAnchor, &mut Node, &mut Visibility),
        With<PlayerCalloutRoot>,
    >,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for (anchor, mut node, mut visibility) in &mut callout_query {
        let Ok(player_transform) = player_query.get(anchor.player) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let head_world_position =
            player_transform.translation + Vec3::Y * crate::core::PLAYER_CALLOUT_HEIGHT_OFFSET;
        let Ok(screen_position) = camera.world_to_viewport(camera_transform, head_world_position)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        node.left = Val::Px(screen_position.x - crate::core::PLAYER_CALLOUT_SCREEN_WIDTH * 0.5);
        node.top = Val::Px(screen_position.y - crate::core::PLAYER_CALLOUT_SCREEN_Y_OFFSET);
        *visibility = Visibility::Visible;
    }
}

pub fn update_player_callout_visuals(
    time: Res<Time>,
    selected_target: Res<super::components::SelectedPassTarget>,
    player_query: Query<(&PlayerDisplayName, &PlayerCallForBall), With<Player>>,
    mut callout_query: Query<
        (
            &PlayerCalloutAnchor,
            &PlayerCalloutParts,
            &mut BackgroundColor,
            &mut Node,
        ),
        With<PlayerCalloutRoot>,
    >,
    mut name_text_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<PlayerCalloutNameText>,
            Without<PlayerCalloutPromptText>,
        ),
    >,
    mut prompt_text_query: Query<
        (&mut Visibility, &mut TextColor),
        (
            With<PlayerCalloutPromptText>,
            Without<PlayerCalloutNameText>,
        ),
    >,
) {
    let pulse = ((time.elapsed_secs() * crate::core::PLAYER_CALLOUT_PULSE_SPEED).sin() * 0.5 + 0.5)
        .clamp(0.0, 1.0);

    for (anchor, parts, mut background_color, mut node) in &mut callout_query {
        let Ok((display_name, call_for_ball)) = player_query.get(anchor.player) else {
            continue;
        };
        let is_selected_target = selected_target.entity == Some(anchor.player);

        if let Ok((mut name_text, mut name_color)) = name_text_query.get_mut(parts.name_text) {
            **name_text = display_name.0.clone();
            let base_name_color = if call_for_ball.active {
                color_from_tuple(crate::core::PLAYER_CALLOUT_CALL_NAME_COLOR)
            } else if is_selected_target {
                color_from_tuple(crate::core::PLAYER_CALLOUT_SELECTED_NAME_COLOR)
            } else {
                color_from_tuple(crate::core::PLAYER_CALLOUT_NORMAL_NAME_COLOR)
            };
            let pulsed_name_color = if call_for_ball.active {
                base_name_color.with_luminance(0.65 + pulse * 0.22)
            } else {
                base_name_color
            };
            *name_color = TextColor(pulsed_name_color);
        }

        if let Ok((mut prompt_visibility, mut prompt_color)) =
            prompt_text_query.get_mut(parts.prompt_text)
        {
            if call_for_ball.active {
                *prompt_visibility = Visibility::Visible;
                *prompt_color = TextColor(
                    color_from_tuple(crate::core::PLAYER_CALLOUT_PROMPT_COLOR)
                        .with_luminance(0.62 + pulse * 0.24),
                );
            } else {
                *prompt_visibility = Visibility::Hidden;
                *prompt_color =
                    TextColor(color_from_tuple(crate::core::PLAYER_CALLOUT_PROMPT_COLOR));
            }
        }

        if call_for_ball.active {
            *background_color = BackgroundColor(Color::srgba(
                0.20,
                0.15 + pulse * 0.12,
                0.04,
                crate::core::PLAYER_CALLOUT_CALL_BG_ALPHA,
            ));
            node.width = Val::Px(crate::core::PLAYER_CALLOUT_SCREEN_WIDTH + pulse * 10.0);
        } else if is_selected_target {
            *background_color = BackgroundColor(Color::srgba(
                0.10,
                0.18 + pulse * 0.06,
                0.24,
                crate::core::PLAYER_CALLOUT_NORMAL_BG_ALPHA + 0.10,
            ));
            node.width = Val::Px(crate::core::PLAYER_CALLOUT_SCREEN_WIDTH + 6.0);
        } else {
            *background_color = BackgroundColor(Color::srgba(
                0.03,
                0.05,
                0.08,
                crate::core::PLAYER_CALLOUT_NORMAL_BG_ALPHA,
            ));
            node.width = Val::Px(crate::core::PLAYER_CALLOUT_SCREEN_WIDTH);
        }
    }
}

fn color_from_tuple((r, g, b): (f32, f32, f32)) -> Color {
    Color::srgb(r, g, b)
}
