use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::features::ritual::patterns::{PossessionJuggleRhythmState, TouchPatternState};
use crate::features::ritual::rhythm::EleganceMeter;
use crate::features::ritual::rules::touch_base_reward;

pub fn update_elegance(
    mut touched_reader: MessageReader<crate::core::BallTouchedEvent>,
    mut pass_resolution_reader: MessageReader<crate::core::PassResolutionEvent>,
    mut whiffed_reader: MessageReader<crate::core::BallWhiffedEvent>,
    mut grounded_reader: MessageReader<crate::core::BallHitGroundEvent>,
    mut pattern_state: ResMut<TouchPatternState>,
    mut juggle_rhythm_state: ResMut<PossessionJuggleRhythmState>,
    mut elegance: ResMut<EleganceMeter>,
) {
    let mut delta = 0.0f32;
    let mut pass_multipliers: HashMap<Entity, VecDeque<f32>> = HashMap::new();

    for pass_resolution in pass_resolution_reader.read() {
        let passer = pass_resolution.passer;
        let _expected_target = pass_resolution.expected_target;
        let _accuracy = pass_resolution.accuracy;
        pass_multipliers
            .entry(pass_resolution.receiver)
            .or_default()
            .push_back(pass_resolution.elegance_multiplier);

        if juggle_rhythm_state.holder == Some(passer) {
            delta -= under_target_possession_penalty(juggle_rhythm_state.juggle_count);
            juggle_rhythm_state.holder = Some(pass_resolution.receiver);
            juggle_rhythm_state.juggle_count = 0;
        }
    }

    for touched in touched_reader.read() {
        let player = touched.player;
        if juggle_rhythm_state.holder != Some(player) {
            if juggle_rhythm_state.holder.is_some() {
                delta -= under_target_possession_penalty(juggle_rhythm_state.juggle_count);
            }
            juggle_rhythm_state.holder = Some(player);
            juggle_rhythm_state.juggle_count = 0;
        }

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
        let pass_multiplier = pass_multipliers
            .get_mut(&player)
            .and_then(|queue| queue.pop_front())
            .unwrap_or(1.0);
        let mut touch_value =
            base + variety_bonus - repeat_penalty - panic_penalty - too_high_penalty;

        if touched.kind == crate::core::TouchKind::Juggle {
            juggle_rhythm_state.juggle_count = juggle_rhythm_state.juggle_count.saturating_add(1);
            let juggle_count = juggle_rhythm_state.juggle_count;
            touch_value *= juggle_gain_multiplier(juggle_count);
        }

        delta += touch_value * quality_scale * pass_multiplier;

        if touched.kind == crate::core::TouchKind::Juggle
            && juggle_rhythm_state.juggle_count > crate::core::ELEGANCE_JUGGLES_MAX_TARGET
        {
            let overflow = (juggle_rhythm_state.juggle_count
                - crate::core::ELEGANCE_JUGGLES_MAX_TARGET) as f32;
            delta -= overflow * crate::core::ELEGANCE_JUGGLE_OVER_TARGET_PENALTY_STEP;
        }

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
        juggle_rhythm_state.holder = None;
        juggle_rhythm_state.juggle_count = 0;
    }

    elegance.value = (elegance.value + delta).clamp(0.0, crate::core::ELEGANCE_MAX);
}

pub fn reset_match_ritual_state(
    mut pattern_state: ResMut<TouchPatternState>,
    mut juggle_rhythm_state: ResMut<PossessionJuggleRhythmState>,
    mut elegance: ResMut<EleganceMeter>,
) {
    *pattern_state = TouchPatternState::default();
    *juggle_rhythm_state = PossessionJuggleRhythmState::default();
    *elegance = EleganceMeter::default();
}

fn under_target_possession_penalty(juggle_count: u8) -> f32 {
    if juggle_count >= crate::core::ELEGANCE_JUGGLES_MIN_TARGET {
        0.0
    } else {
        (crate::core::ELEGANCE_JUGGLES_MIN_TARGET - juggle_count) as f32
            * crate::core::ELEGANCE_JUGGLE_UNDER_TARGET_PENALTY_STEP
    }
}

fn juggle_gain_multiplier(juggle_count: u8) -> f32 {
    if (crate::core::ELEGANCE_JUGGLES_SWEET_SPOT_MIN..=crate::core::ELEGANCE_JUGGLES_SWEET_SPOT_MAX)
        .contains(&juggle_count)
    {
        return 1.45;
    }

    match juggle_count {
        1 => 0.85,
        2 => 1.00,
        3 => 1.10,
        6 => 0.95,
        _ => 0.75,
    }
}
