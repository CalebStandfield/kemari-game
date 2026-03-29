use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::features::ritual::patterns::TouchPatternState;
use crate::features::ritual::rhythm::EleganceMeter;
use crate::features::ritual::rules::touch_base_reward;

pub fn update_elegance(
    mut touched_reader: MessageReader<crate::core::BallTouchedEvent>,
    mut whiffed_reader: MessageReader<crate::core::BallWhiffedEvent>,
    mut grounded_reader: MessageReader<crate::core::BallHitGroundEvent>,
    mut pattern_state: ResMut<TouchPatternState>,
    mut elegance: ResMut<EleganceMeter>,
) {
    let mut delta = 0.0f32;

    for touched in touched_reader.read() {
        let _player = touched.player;
        let base = touch_base_reward(touched.kind);
        let same_as_last = pattern_state.last_touch == Some(touched.kind);
        let variety_bonus = if same_as_last { 0.0 } else { 0.7 };
        let repeat_penalty = if same_as_last {
            pattern_state.repeat_count as f32 * 0.8
        } else {
            0.0
        };
        let panic_penalty = if touched.ball_height < crate::core::ELEGANCE_PANIC_HEIGHT {
            1.1
        } else {
            0.0
        };
        let too_high_penalty = if touched.ball_height > crate::core::ELEGANCE_TOO_HIGH_HEIGHT {
            0.7
        } else {
            0.0
        };
        let quality_scale = 0.6 + touched.quality * 0.4;

        delta += (base + variety_bonus - repeat_penalty - panic_penalty - too_high_penalty)
            * quality_scale;

        if same_as_last {
            pattern_state.repeat_count = pattern_state.repeat_count.saturating_add(1);
        } else {
            pattern_state.last_touch = Some(touched.kind);
            pattern_state.repeat_count = 0;
        }
    }

    for whiffed in whiffed_reader.read() {
        let _player = whiffed.player;
        let panic_whiff_penalty = if whiffed.kind == crate::core::TouchKind::Kick {
            1.4
        } else {
            1.9
        };
        delta -= panic_whiff_penalty;
    }

    for _ in grounded_reader.read() {
        delta -= 4.5;
        pattern_state.last_touch = None;
        pattern_state.repeat_count = 0;
    }

    elegance.value = (elegance.value + delta).clamp(0.0, crate::core::ELEGANCE_MAX);
}
