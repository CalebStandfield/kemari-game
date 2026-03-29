use bevy::prelude::*;

use crate::features::ritual::EleganceMeter;
use crate::features::scoring::ChainCounter;
use crate::features::ui::hud::{ChainText, EleganceText};

pub fn update_gameplay_hud(
    chain: Res<ChainCounter>,
    elegance: Res<EleganceMeter>,
    mut chain_text_query: Query<&mut Text, (With<ChainText>, Without<EleganceText>)>,
    mut elegance_text_query: Query<&mut Text, (With<EleganceText>, Without<ChainText>)>,
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
}
