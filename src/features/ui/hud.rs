use bevy::prelude::*;
use bevy::ui::{px, Val};

#[derive(Component)]
pub struct GameplayHudRoot;

#[derive(Component)]
pub struct ChainText;

#[derive(Component)]
pub struct EleganceText;

#[derive(Component)]
pub struct PassQueueText;

#[derive(Component)]
pub struct SelectedTargetText;

pub fn spawn_gameplay_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.72)),
            GameplayHudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Chain: 0"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.96, 0.95, 0.86)),
                ChainText,
            ));
            parent.spawn((
                Text::new("Elegance: 50.0"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.83, 0.90, 0.97)),
                EleganceText,
            ));
            parent.spawn((
                Text::new("Pass Queue:\n(empty)"),
                TextFont::from_font_size(17.0),
                TextColor(Color::srgb(0.88, 0.91, 0.96)),
                PassQueueText,
            ));
            parent.spawn((
                Text::new("Selected Target: (none)"),
                TextFont::from_font_size(17.0),
                TextColor(Color::srgb(0.68, 0.84, 1.0)),
                SelectedTargetText,
            ));
        });
}

pub fn despawn_gameplay_hud(
    mut commands: Commands,
    root_query: Query<Entity, With<GameplayHudRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }
}
