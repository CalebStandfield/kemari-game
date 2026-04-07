use bevy::prelude::*;

use crate::features::player::{PlayerDisplayName, PlayerPassRequestQueue, SelectedPassTarget};
use crate::features::ritual::EleganceMeter;
use crate::features::scoring::ChainCounter;
use crate::features::ui::hud::{ChainText, EleganceText, PassQueueText, SelectedTargetText};

pub fn update_gameplay_hud(
    chain: Res<ChainCounter>,
    elegance: Res<EleganceMeter>,
    pass_queue: Res<PlayerPassRequestQueue>,
    selected_target: Res<SelectedPassTarget>,
    player_names: Query<&PlayerDisplayName>,
    mut chain_text_query: Query<
        &mut Text,
        (
            With<ChainText>,
            Without<EleganceText>,
            Without<PassQueueText>,
            Without<SelectedTargetText>,
        ),
    >,
    mut elegance_text_query: Query<
        &mut Text,
        (
            With<EleganceText>,
            Without<ChainText>,
            Without<PassQueueText>,
            Without<SelectedTargetText>,
        ),
    >,
    mut pass_queue_text_query: Query<
        &mut Text,
        (
            With<PassQueueText>,
            Without<ChainText>,
            Without<EleganceText>,
            Without<SelectedTargetText>,
        ),
    >,
    mut selected_target_text_query: Query<
        &mut Text,
        (
            With<SelectedTargetText>,
            Without<ChainText>,
            Without<EleganceText>,
            Without<PassQueueText>,
        ),
    >,
) {
    if chain.is_changed() {
        for mut text in &mut chain_text_query {
            **text = format!("Chain: {}", chain.value);
        }
    }

    if elegance.is_changed() {
        for mut text in &mut elegance_text_query {
            **text = format!("Elegance: {:.1}", elegance.value);
        }
    }

    let mut lines = String::from("Pass Queue:");
    if pass_queue.order.is_empty() {
        lines.push_str("\n(empty)");
    } else {
        for (index, player_entity) in pass_queue.order.iter().enumerate() {
            let name = player_names
                .get(*player_entity)
                .map(|display_name| display_name.0.as_str())
                .unwrap_or("Unknown");
            lines.push_str(&format!("\n{}. {}", index + 1, name));
        }
    }

    for mut text in &mut pass_queue_text_query {
        **text = lines.clone();
    }

    let selected_target_line = selected_target
        .entity
        .and_then(|entity| player_names.get(entity).ok())
        .map(|display_name| format!("Selected Target: {}", display_name.0))
        .unwrap_or_else(|| "Selected Target: (none)".to_string());

    for mut text in &mut selected_target_text_query {
        **text = selected_target_line.clone();
    }
}
