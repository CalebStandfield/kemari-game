use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::log::debug;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use super::components::{ControlledPlayer, Player, PlayerCallForBall};

#[derive(Resource, Debug, Default)]
pub struct PlayerPassRequestQueue {
    pub order: VecDeque<Entity>,
    pub npc_rejoin_cooldowns: HashMap<Entity, f32>,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct BallPossessionState {
    pub holder: Option<Entity>,
}

#[derive(Resource, Debug, Default)]
pub struct PassQueueDebugState {
    pub last_reject_reason: HashMap<Entity, PassQueueRejectReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassQueueRejectReason {
    AlreadyQueued,
    HasBallControl,
    NpcCooldownActive,
    QueueFull,
}

impl PlayerPassRequestQueue {
    pub fn can_join_pass_queue(
        &self,
        player: Entity,
        is_human: bool,
        has_ball_control: bool,
    ) -> Result<(), PassQueueRejectReason> {
        if has_ball_control {
            return Err(PassQueueRejectReason::HasBallControl);
        }
        if self.order.contains(&player) {
            return Err(PassQueueRejectReason::AlreadyQueued);
        }

        if !is_human {
            if self
                .npc_rejoin_cooldowns
                .get(&player)
                .is_some_and(|remaining| *remaining > 0.0)
            {
                return Err(PassQueueRejectReason::NpcCooldownActive);
            }

            if self.order.len() >= crate::core::PLAYER_PASS_REQUEST_QUEUE_CAPACITY {
                return Err(PassQueueRejectReason::QueueFull);
            }
        }

        Ok(())
    }

    pub fn enqueue_pass_request(
        &mut self,
        player: Entity,
        is_human: bool,
        has_ball_control: bool,
    ) -> Result<(), PassQueueRejectReason> {
        self.can_join_pass_queue(player, is_human, has_ball_control)?;
        self.order.push_back(player);
        Ok(())
    }

    pub fn remove_pass_request(&mut self, player: Entity) -> bool {
        if let Some(index) = self.order.iter().position(|queued| *queued == player) {
            self.order.remove(index);
            return true;
        }
        false
    }
}

pub fn tick_npc_rejoin_cooldowns(time: Res<Time>, mut queue: ResMut<PlayerPassRequestQueue>) {
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    queue.npc_rejoin_cooldowns.retain(|_, remaining| {
        *remaining = (*remaining - delta_seconds).max(0.0);
        *remaining > 0.0
    });
}

pub fn sync_queue_from_call_state(
    mut queue: ResMut<PlayerPassRequestQueue>,
    possession: Res<BallPossessionState>,
    mut debug_state: ResMut<PassQueueDebugState>,
    mut player_query: Query<
        (Entity, Option<&ControlledPlayer>, &mut PlayerCallForBall),
        With<Player>,
    >,
) {
    for (player, controlled, mut call_for_ball) in &mut player_query {
        let is_human = controlled.is_some();
        let has_ball_control = possession.holder == Some(player);

        if has_ball_control {
            if call_for_ball.active {
                call_for_ball.active = false;
                debug!(
                    "pass_queue: player {:?} call state cleared because they control the ball",
                    player
                );
            }
            if queue.remove_pass_request(player) {
                debug!(
                    "pass_queue: removed player {:?} because they control the ball",
                    player
                );
            }
            debug_state.last_reject_reason.remove(&player);
            continue;
        }

        if !call_for_ball.active {
            if queue.remove_pass_request(player) {
                debug!(
                    "pass_queue: removed player {:?} because they stopped calling",
                    player
                );
            }
            debug_state.last_reject_reason.remove(&player);
            continue;
        }

        if queue.order.contains(&player) {
            debug_state.last_reject_reason.remove(&player);
            continue;
        }

        match queue.enqueue_pass_request(player, is_human, has_ball_control) {
            Ok(()) => {
                debug!(
                    "pass_queue: enqueued player {:?} (human: {})",
                    player, is_human
                );
                debug_state.last_reject_reason.remove(&player);
            }
            Err(reason) => {
                let previous = debug_state.last_reject_reason.insert(player, reason);
                if previous != Some(reason) {
                    debug!(
                        "pass_queue: rejected player {:?} (human: {}) because {:?}",
                        player, is_human, reason
                    );
                }
            }
        }
    }
}

pub fn apply_ball_possession_to_queue(
    mut touched_reader: MessageReader<crate::core::BallTouchedEvent>,
    mut grounded_reader: MessageReader<crate::core::BallHitGroundEvent>,
    mut pass_launched_reader: MessageReader<crate::core::BallPassLaunchedEvent>,
    mut pass_resolution_writer: MessageWriter<crate::core::PassResolutionEvent>,
    mut queue: ResMut<PlayerPassRequestQueue>,
    mut possession: ResMut<BallPossessionState>,
    mut debug_state: ResMut<PassQueueDebugState>,
    controlled_query: Query<(), With<ControlledPlayer>>,
    mut call_state_query: Query<&mut PlayerCallForBall, With<Player>>,
) {
    for touched in touched_reader.read() {
        let player = touched.player;
        let is_human = controlled_query.get(player).is_ok();
        let previous_holder = possession.holder;
        let expected_target = queue.order.front().copied();

        possession.holder = Some(player);

        if let Ok(mut call_for_ball) = call_state_query.get_mut(player) {
            call_for_ball.active = false;
        }

        let was_queued = queue.remove_pass_request(player);
        if was_queued {
            debug!(
                "pass_queue: player {:?} removed after receiving the ball",
                player
            );
        }

        debug_state.last_reject_reason.remove(&player);

        if was_queued && !is_human {
            queue
                .npc_rejoin_cooldowns
                .insert(player, crate::core::PLAYER_QUEUE_TIME_WAIT);
            debug!(
                "pass_queue: npc {:?} cooldown started for {:.2}s",
                player,
                crate::core::PLAYER_QUEUE_TIME_WAIT
            );
        }

        if let Some(passer) = previous_holder.filter(|previous| *previous != player) {
            let (accuracy, elegance_multiplier) = match expected_target {
                Some(expected) if expected == player => {
                    (crate::core::PassTargetAccuracy::CorrectQueueTarget, 1.25)
                }
                Some(_) => (crate::core::PassTargetAccuracy::IncorrectQueueTarget, 0.75),
                None => (crate::core::PassTargetAccuracy::NoQueueTarget, 1.0),
            };

            pass_resolution_writer.write(crate::core::PassResolutionEvent {
                passer,
                receiver: player,
                expected_target,
                accuracy,
                elegance_multiplier,
            });

            debug!(
                "pass_queue: pass {:?} -> {:?}, expected {:?}, {:?}, x{:.2}",
                passer, player, expected_target, accuracy, elegance_multiplier
            );
        }
    }

    for pass_launched in pass_launched_reader.read() {
        if possession.holder == Some(pass_launched.passer) {
            possession.holder = None;
            debug!(
                "pass_queue: possession released by pass {:?} -> {:?} ({:?})",
                pass_launched.passer, pass_launched.receiver, pass_launched.kind
            );
        }
    }

    if grounded_reader.read().next().is_some() {
        possession.holder = None;
    }
}

pub fn prune_invalid_queue_members(
    mut queue: ResMut<PlayerPassRequestQueue>,
    possession: Res<BallPossessionState>,
    mut debug_state: ResMut<PassQueueDebugState>,
    player_query: Query<(Entity, Option<&ControlledPlayer>, &PlayerCallForBall), With<Player>>,
) {
    let mut player_state = HashMap::new();
    for (player, controlled, call_for_ball) in &player_query {
        player_state.insert(player, (controlled.is_some(), call_for_ball.active));
    }
    let cooldown_snapshot = queue.npc_rejoin_cooldowns.clone();

    queue.order.retain(|player| {
        let Some((is_human, is_calling)) = player_state.get(player).copied() else {
            debug!(
                "pass_queue: removed stale player {:?} that no longer exists",
                player
            );
            debug_state.last_reject_reason.remove(player);
            return false;
        };

        if !is_calling {
            debug!(
                "pass_queue: removed player {:?} because they are not calling",
                player
            );
            debug_state.last_reject_reason.remove(player);
            return false;
        }

        if possession.holder == Some(*player) {
            debug!(
                "pass_queue: removed player {:?} because they control the ball",
                player
            );
            debug_state.last_reject_reason.remove(player);
            return false;
        }

        if !is_human
            && cooldown_snapshot
                .get(player)
                .is_some_and(|remaining| *remaining > 0.0)
        {
            debug!(
                "pass_queue: removed npc {:?} due to active cooldown",
                player
            );
            debug_state.last_reject_reason.remove(player);
            return false;
        }

        true
    });

    queue
        .npc_rejoin_cooldowns
        .retain(|player, _| player_state.contains_key(player));
}

pub fn reset_pass_queue_state(
    mut queue: ResMut<PlayerPassRequestQueue>,
    mut possession: ResMut<BallPossessionState>,
    mut debug_state: ResMut<PassQueueDebugState>,
) {
    queue.order.clear();
    queue.npc_rejoin_cooldowns.clear();
    possession.holder = None;
    debug_state.last_reject_reason.clear();
}
