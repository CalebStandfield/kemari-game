use bevy::prelude::*;
use bevy::ui::{px, Val};

#[derive(Component)]
pub struct GameplayHudRoot;

#[derive(Component)]
pub struct ChainText;

#[derive(Component)]
pub struct EleganceText;

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
