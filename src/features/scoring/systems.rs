use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::features::scoring::chain::ChainCounter;

pub fn update_chain(
    mut touched_reader: MessageReader<crate::core::BallTouchedEvent>,
    mut ground_reader: MessageReader<crate::core::BallHitGroundEvent>,
    mut chain: ResMut<ChainCounter>,
) {
    let mut touched_count = 0u32;
    for _ in touched_reader.read() {
        touched_count += 1;
    }

    let mut touched_ground = false;
    for _ in ground_reader.read() {
        touched_ground = true;
    }

    if touched_ground {
        chain.value = 0;
    }

    if touched_count > 0 {
        chain.value = chain.value.saturating_add(touched_count);
    }
}
