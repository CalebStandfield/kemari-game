mod animation;
mod callout;
mod components;
mod kick;
mod movement;
mod pass_queue;
mod spawn;
mod systems;

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use std::f32::consts::TAU;

pub use components::{
    ControlledPlayer, Player, PlayerDisplayName, PlayerFacing, SelectedPassTarget,
};
pub use pass_queue::{BallPossessionState, PlayerPassRequestQueue};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<pass_queue::PlayerPassRequestQueue>()
            .init_resource::<pass_queue::BallPossessionState>()
            .init_resource::<pass_queue::PassQueueDebugState>()
            .init_resource::<components::SelectedPassTarget>()
            .add_systems(OnEnter(crate::core::GameState::InGame), spawn_players)
            .add_systems(
                OnExit(crate::core::GameState::InGame),
                (despawn_players, reset_selected_pass_target_state),
            )
            .add_systems(
                OnExit(crate::core::GameState::InGame),
                pass_queue::reset_pass_queue_state,
            )
            .add_systems(
                Update,
                (
                    tick_touch_cooldowns,
                    update_selected_pass_target_input,
                    update_controlled_player_call_state,
                    update_npc_behavior_state,
                    pass_queue::tick_npc_rejoin_cooldowns,
                    pass_queue::sync_queue_from_call_state,
                    pass_queue::prune_invalid_queue_members,
                    emit_human_touch_attempts,
                    emit_npc_touch_attempts,
                    apply_zone_movement,
                    resolve_player_collisions,
                )
                    .chain()
                    .in_set(crate::core::GameplaySet::PlayerInput)
                    .run_if(in_state(crate::core::GameState::InGame)),
            )
            .add_systems(
                Update,
                pass_queue::apply_ball_possession_to_queue
                    .in_set(crate::core::GameplaySet::Scoring)
                    .run_if(in_state(crate::core::GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    callout::update_player_callout_positions,
                    callout::update_player_callout_visuals,
                )
                    .chain()
                    .in_set(crate::core::GameplaySet::Ui)
                    .run_if(in_state(crate::core::GameState::InGame)),
            );
    }
}

fn spawn_players(
    mut commands: Commands,
    session_config: Res<crate::core::SessionConfig>,
    asset_server: Res<AssetServer>,
) {
    let player_count = session_config.player_count.clamp(1, 8);
    let ring_radius = if player_count == 1 {
        0.0
    } else {
        crate::core::PLAYER_RING_RADIUS
    };
    let ring_center_x = crate::core::BALL_START_X;
    let ring_center_z = crate::core::BALL_START_Z;
    let player_scene: Handle<Scene> = asset_server.load(crate::core::PLAYER_SCENE_PATH);

    let mut default_target: Option<(usize, Entity)> = None;

    for player_index in 0..player_count {
        let angle = (player_index as f32 / player_count as f32) * TAU;
        let x = ring_center_x + ring_radius * angle.cos();
        let z = ring_center_z + ring_radius * angle.sin();
        let home = Vec2::new(x, z);
        let is_controlled = player_index == 0;
        let slot_index = player_index + 1;
        let display_name = format!("Player {slot_index}");

        let toward_center = Vec2::new(ring_center_x - x, ring_center_z - z).normalize_or_zero();
        let initial_facing = if toward_center == Vec2::ZERO {
            Vec2::new(0.0, -1.0)
        } else {
            toward_center
        };

        let mut transform = Transform::from_xyz(x, crate::core::PLAYER_Y, z);
        if player_count > 1 {
            transform.look_at(
                Vec3::new(ring_center_x, crate::core::PLAYER_Y, ring_center_z),
                Vec3::Y,
            );
        }

        let mut entity_commands = commands.spawn((
            Player,
            crate::core::PlayerBody,
            components::PlayerDisplayName(display_name.clone()),
            components::PlayerSlot { index: slot_index },
            components::PlayerHomePosition(home),
            components::PlayerZoneRadius(crate::core::PLAYER_ZONE_RADIUS),
            components::PlayerDesiredMove::default(),
            components::PlayerCallForBall::default(),
            components::PlayerFacing(initial_facing),
            components::PlayerTouchCooldowns::default(),
            transform,
        ));

        if !is_controlled {
            let phase = slot_index as f32 * 0.173;
            entity_commands.insert(components::NpcBehavior::new(home, phase));
        }

        entity_commands.with_children(|parent| {
            parent.spawn((
                SceneRoot(player_scene.clone()),
                Transform::from_xyz(
                    crate::core::PLAYER_MODEL_OFFSET_X,
                    crate::core::PLAYER_MODEL_OFFSET_Y,
                    crate::core::PLAYER_MODEL_OFFSET_Z,
                )
                .with_rotation(Quat::from_rotation_y(
                    crate::core::PLAYER_MODEL_ROT_Y_DEG.to_radians(),
                ))
                .with_scale(Vec3::splat(crate::core::PLAYER_MODEL_SCALE)),
            ));
        });

        if is_controlled {
            entity_commands.insert(ControlledPlayer);
        }

        let player_entity = entity_commands.id();
        if !is_controlled && default_target.is_none() {
            default_target = Some((slot_index, player_entity));
        }

        callout::spawn_player_callout(&mut commands, player_entity, &display_name);
    }

    let (slot_index, entity) = default_target.unwrap_or((1, Entity::PLACEHOLDER));
    commands.insert_resource(components::SelectedPassTarget {
        slot_index,
        entity: (entity != Entity::PLACEHOLDER).then_some(entity),
    });
}

fn tick_touch_cooldowns(
    time: Res<Time>,
    mut player_query: Query<&mut components::PlayerTouchCooldowns, With<Player>>,
) {
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    for mut cooldowns in &mut player_query {
        cooldowns.kick = (cooldowns.kick - delta_seconds).max(0.0);
        cooldowns.head = (cooldowns.head - delta_seconds).max(0.0);
        cooldowns.juggle = (cooldowns.juggle - delta_seconds).max(0.0);
    }
}

fn update_selected_pass_target_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_target: ResMut<components::SelectedPassTarget>,
    player_query: Query<(Entity, &components::PlayerSlot, Option<&ControlledPlayer>), With<Player>>,
) {
    let mut players = Vec::new();
    for (entity, slot, controlled) in &player_query {
        players.push((slot.index, entity, controlled.is_some()));
    }
    players.sort_by_key(|(slot, _, _)| *slot);

    let controlled_player = players
        .iter()
        .find_map(|(_, entity, is_controlled)| is_controlled.then_some(*entity));

    if let Some(slot_index) = read_selected_target_slot(&keyboard_input) {
        if let Some(entity) = find_target_entity_by_slot(&players, slot_index, controlled_player) {
            selected_target.slot_index = slot_index;
            selected_target.entity = Some(entity);
        }
    }

    let selected_is_valid = selected_target.entity.is_some_and(|entity| {
        Some(entity) != controlled_player
            && players.iter().any(|(_, candidate, _)| *candidate == entity)
    });

    if !selected_is_valid {
        if let Some((slot_index, entity)) = pick_default_target(&players, controlled_player) {
            selected_target.slot_index = slot_index;
            selected_target.entity = Some(entity);
        } else {
            selected_target.entity = None;
        }
    }
}

fn update_controlled_player_call_state(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    possession: Res<pass_queue::BallPossessionState>,
    mut player_query: Query<(Entity, &mut components::PlayerCallForBall), With<ControlledPlayer>>,
) {
    let toggle_call = keyboard_input.just_pressed(KeyCode::KeyL);

    for (player, mut call_for_ball) in &mut player_query {
        if possession.holder == Some(player) {
            call_for_ball.active = false;
            continue;
        }

        if toggle_call {
            call_for_ball.active = !call_for_ball.active;
        }
    }
}

fn update_npc_behavior_state(
    time: Res<Time>,
    possession: Res<pass_queue::BallPossessionState>,
    ball_query: Query<
        (&Transform, &crate::features::ball::BallIncomingPass),
        With<crate::features::ball::Ball>,
    >,
    mut npc_query: Query<
        (
            Entity,
            &Transform,
            &components::PlayerSlot,
            &components::PlayerHomePosition,
            &components::PlayerZoneRadius,
            &mut components::PlayerCallForBall,
            &mut components::PlayerDesiredMove,
            &mut components::PlayerFacing,
            &mut components::NpcBehavior,
        ),
        (With<Player>, Without<ControlledPlayer>),
    >,
) {
    let (ball_position, incoming_receiver) =
        if let Ok((ball_transform, incoming_pass)) = ball_query.single() {
            (
                Vec2::new(ball_transform.translation.x, ball_transform.translation.z),
                incoming_pass.receiver,
            )
        } else {
            (Vec2::ZERO, None)
        };

    for (
        entity,
        transform,
        slot,
        home,
        zone,
        mut call_for_ball,
        mut desired_move,
        mut facing,
        mut behavior,
    ) in &mut npc_query
    {
        behavior.drift_timer.tick(time.delta());
        behavior.call_decision_timer.tick(time.delta());
        behavior.pass_timer.tick(time.delta());

        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let to_home = home.0 - position;
        let home_distance = to_home.length();
        let has_ball_control = possession.holder == Some(entity);
        let is_intended_receiver = incoming_receiver == Some(entity);

        if has_ball_control {
            if behavior.state != components::NpcBehaviorState::ControllingBall {
                let phase = time.elapsed_secs() * 0.6 + slot.index as f32 * 0.77;
                let next_delay = components::pass_hesitation_from_phase(phase);
                behavior
                    .pass_timer
                    .set_duration(std::time::Duration::from_secs_f32(next_delay));
                behavior.pass_timer.reset();
            }
            behavior.state = components::NpcBehaviorState::ControllingBall;
            call_for_ball.active = false;
            desired_move.0 = to_home.normalize_or_zero() * crate::core::PLAYER_ZONE_IDLE_SPEED;

            let face_ball = (ball_position - position).normalize_or_zero();
            if face_ball.length_squared() > 0.0 {
                facing.0 = face_ball;
            }
            continue;
        }

        if is_intended_receiver {
            behavior.state = components::NpcBehaviorState::PreparingToReceive;
            call_for_ball.active = true;

            let toward_ball = (ball_position - position).normalize_or_zero();
            let prep_target = clamp_to_zone(
                home.0,
                home.0 + toward_ball * crate::core::NPC_RECEIVE_PREP_RADIUS,
                zone.0,
            );
            let to_prep = prep_target - position;
            desired_move.0 = to_prep.normalize_or_zero() * crate::core::PLAYER_ZONE_RECEIVE_SPEED;

            if toward_ball.length_squared() > 0.0 {
                facing.0 = toward_ball;
            }
            continue;
        }

        if home_distance > zone.0 * 0.82 {
            behavior.state = components::NpcBehaviorState::RecoveringToHome;
            call_for_ball.active = false;
            desired_move.0 = to_home.normalize_or_zero() * crate::core::PLAYER_ZONE_RETURN_SPEED;
            if to_home.length_squared() > 0.0 {
                facing.0 = to_home.normalize();
            }
            continue;
        }

        if behavior.call_decision_timer.just_finished() {
            let signal_phase = time.elapsed_secs() * 0.9 + slot.index as f32 * 1.37;
            let call_signal = (signal_phase.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            call_for_ball.active = call_signal > crate::core::NPC_CALL_SIGNAL_THRESHOLD;
        }

        behavior.state = if call_for_ball.active {
            components::NpcBehaviorState::CallingForPass
        } else {
            components::NpcBehaviorState::Idle
        };

        if behavior.drift_timer.just_finished()
            || (behavior.drift_target - position).length()
                <= crate::core::PLAYER_ZONE_WANDER_REACHED_DISTANCE
        {
            let drift_phase = time.elapsed_secs() * 0.58 + slot.index as f32 * 1.73;
            behavior.drift_target = compute_zone_drift_target(home.0, zone.0, drift_phase);

            if call_for_ball.active {
                let toward_ball = (ball_position - home.0).normalize_or_zero();
                behavior.drift_target = clamp_to_zone(
                    home.0,
                    behavior.drift_target + toward_ball * crate::core::PLAYER_CALL_STEP_BIAS,
                    zone.0,
                );
            }
        }

        let toward_drift = behavior.drift_target - position;
        let speed = if call_for_ball.active {
            crate::core::PLAYER_ZONE_RECEIVE_SPEED
        } else {
            crate::core::PLAYER_ZONE_IDLE_SPEED
        };
        desired_move.0 = toward_drift.normalize_or_zero() * speed;

        let look_vector = (ball_position - position).normalize_or_zero();
        if look_vector.length_squared() > 0.0 {
            facing.0 = look_vector;
        }
    }
}

fn emit_human_touch_attempts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_target: Res<components::SelectedPassTarget>,
    pass_queue: Res<pass_queue::PlayerPassRequestQueue>,
    mut touch_writer: MessageWriter<crate::core::PlayerTouchAttemptEvent>,
    mut controlled_query: Query<
        (
            Entity,
            &components::PlayerFacing,
            &mut components::PlayerTouchCooldowns,
        ),
        With<ControlledPlayer>,
    >,
    player_query: Query<(Entity, Option<&ControlledPlayer>, &components::PlayerSlot), With<Player>>,
) {
    let Ok((player_entity, facing, mut cooldowns)) = controlled_query.single_mut() else {
        return;
    };

    let mut players = Vec::new();
    for (entity, controlled, slot) in &player_query {
        players.push((slot.index, entity, controlled.is_some()));
    }
    players.sort_by_key(|(slot, _, _)| *slot);

    let chosen_target = resolve_preferred_pass_target(
        player_entity,
        selected_target.entity,
        &pass_queue.order,
        &players,
    );

    let facing = facing.0.normalize_or_zero();
    if keyboard_input.just_pressed(KeyCode::KeyK) && cooldowns.kick <= 0.0 {
        cooldowns.kick = crate::core::TOUCH_COOLDOWN_KICK;
        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: player_entity,
            kind: crate::core::TouchKind::Kick,
            facing,
            target: chosen_target,
        });
    }

    if keyboard_input.just_pressed(KeyCode::KeyH) && cooldowns.head <= 0.0 {
        cooldowns.head = crate::core::TOUCH_COOLDOWN_HEAD;
        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: player_entity,
            kind: crate::core::TouchKind::Head,
            facing,
            target: chosen_target,
        });
    }

    if keyboard_input.just_pressed(KeyCode::KeyJ) && cooldowns.juggle <= 0.0 {
        cooldowns.juggle = crate::core::TOUCH_COOLDOWN_JUGGLE;
        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: player_entity,
            kind: crate::core::TouchKind::Juggle,
            facing,
            target: chosen_target,
        });
    }
}

fn emit_npc_touch_attempts(
    time: Res<Time>,
    possession: Res<pass_queue::BallPossessionState>,
    pass_queue: Res<pass_queue::PlayerPassRequestQueue>,
    ball_query: Query<&Transform, With<crate::features::ball::Ball>>,
    mut touch_writer: MessageWriter<crate::core::PlayerTouchAttemptEvent>,
    mut npc_query: Query<
        (
            Entity,
            &Transform,
            &components::PlayerSlot,
            &components::PlayerFacing,
            &mut components::PlayerTouchCooldowns,
            &mut components::NpcBehavior,
        ),
        (With<Player>, Without<ControlledPlayer>),
    >,
    candidate_query: Query<(Entity, &Transform, &components::PlayerCallForBall), With<Player>>,
) {
    let ball_height = ball_query
        .single()
        .map(|transform| transform.translation.y)
        .unwrap_or(crate::core::BALL_RADIUS);

    let mut candidates = Vec::new();
    for (entity, transform, call_for_ball) in &candidate_query {
        candidates.push((
            entity,
            Vec2::new(transform.translation.x, transform.translation.z),
            call_for_ball.active,
        ));
    }

    for (entity, transform, slot, facing, mut cooldowns, mut behavior) in &mut npc_query {
        if possession.holder != Some(entity) {
            continue;
        }
        if !behavior.pass_timer.is_finished() {
            continue;
        }

        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let maybe_target = choose_npc_target(
            entity,
            position,
            behavior.last_pass_target,
            &pass_queue.order,
            &candidates,
        );

        let Some(target) = maybe_target else {
            behavior.pass_timer.reset();
            continue;
        };

        let target_position = candidates
            .iter()
            .find_map(|(candidate, pos, _)| (*candidate == target).then_some(*pos))
            .unwrap_or(position);

        let to_target = target_position - position;
        let distance = to_target.length();
        let phase = (time.elapsed_secs() * 0.6 + slot.index as f32 * 0.77).sin() * 0.5 + 0.5;
        let touch_kind = choose_npc_touch_kind(distance, ball_height, phase);

        let cooldown = match touch_kind {
            crate::core::TouchKind::Kick => &mut cooldowns.kick,
            crate::core::TouchKind::Head => &mut cooldowns.head,
            crate::core::TouchKind::Juggle => &mut cooldowns.juggle,
        };

        if *cooldown > 0.0 {
            continue;
        }

        *cooldown = match touch_kind {
            crate::core::TouchKind::Kick => crate::core::TOUCH_COOLDOWN_KICK,
            crate::core::TouchKind::Head => crate::core::TOUCH_COOLDOWN_HEAD,
            crate::core::TouchKind::Juggle => crate::core::TOUCH_COOLDOWN_JUGGLE,
        };

        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: entity,
            kind: touch_kind,
            facing: facing.0,
            target: Some(target),
        });

        behavior.state = components::NpcBehaviorState::Passing;
        behavior.last_pass_target = Some(target);
        let next_delay = components::pass_hesitation_from_phase(phase + slot.index as f32 * 0.11);
        behavior
            .pass_timer
            .set_duration(std::time::Duration::from_secs_f32(next_delay));
        behavior.pass_timer.reset();
    }
}

fn apply_zone_movement(
    time: Res<Time>,
    mut player_query: Query<
        (
            &mut Transform,
            &components::PlayerHomePosition,
            &components::PlayerZoneRadius,
            &components::PlayerDesiredMove,
            &mut components::PlayerFacing,
            Option<&ControlledPlayer>,
        ),
        With<Player>,
    >,
) {
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let turn_lerp = (crate::core::PLAYER_TURN_SPEED * delta_seconds).clamp(0.0, 1.0);

    for (mut transform, home, zone, desired_move, mut facing, controlled) in &mut player_query {
        let current = Vec2::new(transform.translation.x, transform.translation.z);
        let to_home = home.0 - current;

        let mut velocity = desired_move.0;
        if to_home.length() > zone.0 {
            velocity += to_home.normalize_or_zero() * crate::core::PLAYER_ZONE_RETURN_SPEED;
        }

        if controlled.is_some() {
            velocity *= 0.65;
        }

        let mut step = velocity * delta_seconds;
        let max_step = crate::core::PLAYER_MAX_MOVE_PER_FRAME;
        if step.length() > max_step {
            step = step.normalize() * max_step;
        }

        let mut next = current + step;
        next = clamp_to_zone(home.0, next, zone.0);
        next.x = next.x.clamp(-max_x, max_x);
        next.y = next.y.clamp(-max_z, max_z);

        transform.translation.x = next.x;
        transform.translation.z = next.y;
        transform.translation.y = crate::core::PLAYER_Y;

        let move_dir = velocity.normalize_or_zero();
        if move_dir.length_squared() > 0.0 {
            facing.0 = move_dir;
            let facing_direction = Vec3::new(move_dir.x, 0.0, move_dir.y);
            let target_rotation = Transform::IDENTITY
                .looking_to(facing_direction, Vec3::Y)
                .rotation;
            transform.rotation = transform.rotation.slerp(target_rotation, turn_lerp);
        }
    }
}

fn resolve_player_collisions(mut player_query: Query<&mut Transform, With<Player>>) {
    let min_distance = crate::core::PLAYER_COLLIDER_RADIUS * 2.0;
    let min_distance_squared = min_distance * min_distance;
    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;

    for _ in 0..2 {
        let mut pairs = player_query.iter_combinations_mut();

        while let Some([mut player_a, mut player_b]) = pairs.fetch_next() {
            let separation = Vec2::new(
                player_a.translation.x - player_b.translation.x,
                player_a.translation.z - player_b.translation.z,
            );
            let separation_squared = separation.length_squared();

            if separation_squared >= min_distance_squared {
                continue;
            }

            let normal = if separation_squared > f32::EPSILON {
                separation / separation_squared.sqrt()
            } else {
                Vec2::X
            };
            let penetration = min_distance - separation_squared.sqrt();
            let correction = normal * (penetration * 0.5);

            player_a.translation.x += correction.x;
            player_a.translation.z += correction.y;
            player_b.translation.x -= correction.x;
            player_b.translation.z -= correction.y;

            player_a.translation.x = player_a.translation.x.clamp(-max_x, max_x);
            player_a.translation.z = player_a.translation.z.clamp(-max_z, max_z);
            player_b.translation.x = player_b.translation.x.clamp(-max_x, max_x);
            player_b.translation.z = player_b.translation.z.clamp(-max_z, max_z);
            player_a.translation.y = crate::core::PLAYER_Y;
            player_b.translation.y = crate::core::PLAYER_Y;
        }
    }
}

fn despawn_players(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    callout_query: Query<Entity, With<callout::PlayerCalloutRoot>>,
) {
    for entity in &callout_query {
        commands.entity(entity).despawn();
    }

    for entity in &player_query {
        commands.entity(entity).despawn();
    }
}

fn reset_selected_pass_target_state(mut selected_target: ResMut<components::SelectedPassTarget>) {
    *selected_target = components::SelectedPassTarget::default();
}

fn read_selected_target_slot(keyboard_input: &ButtonInput<KeyCode>) -> Option<usize> {
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

fn find_target_entity_by_slot(
    players: &[(usize, Entity, bool)],
    slot_index: usize,
    controlled_player: Option<Entity>,
) -> Option<Entity> {
    players.iter().find_map(|(slot, entity, _)| {
        (*slot == slot_index && Some(*entity) != controlled_player).then_some(*entity)
    })
}

fn pick_default_target(
    players: &[(usize, Entity, bool)],
    controlled_player: Option<Entity>,
) -> Option<(usize, Entity)> {
    players.iter().find_map(|(slot, entity, _)| {
        (Some(*entity) != controlled_player).then_some((*slot, *entity))
    })
}

fn resolve_preferred_pass_target(
    passer: Entity,
    selected_target: Option<Entity>,
    queue: &std::collections::VecDeque<Entity>,
    players: &[(usize, Entity, bool)],
) -> Option<Entity> {
    if let Some(target) = selected_target.filter(|target| *target != passer) {
        if players.iter().any(|(_, entity, _)| *entity == target) {
            return Some(target);
        }
    }

    if let Some(target) = queue
        .iter()
        .copied()
        .find(|queued_player| *queued_player != passer)
    {
        return Some(target);
    }

    players
        .iter()
        .find_map(|(_, entity, _)| (*entity != passer).then_some(*entity))
}

fn choose_npc_target(
    passer: Entity,
    passer_position: Vec2,
    last_target: Option<Entity>,
    queue: &std::collections::VecDeque<Entity>,
    candidates: &[(Entity, Vec2, bool)],
) -> Option<Entity> {
    for queued in queue {
        if *queued != passer && candidates.iter().any(|(entity, _, _)| *entity == *queued) {
            return Some(*queued);
        }
    }

    let mut best: Option<(Entity, f32)> = None;
    for (candidate, position, is_calling) in candidates {
        if *candidate == passer {
            continue;
        }

        let distance = (*position - passer_position).length();
        let mut score = if *is_calling { 3.0 } else { 1.0 };
        score -= distance * 0.14;

        if Some(*candidate) == last_target {
            score -= 1.35;
        }

        if let Some((_, best_score)) = best {
            if score > best_score {
                best = Some((*candidate, score));
            }
        } else {
            best = Some((*candidate, score));
        }
    }

    best.map(|(target, _)| target)
}

fn choose_npc_touch_kind(distance: f32, ball_height: f32, phase: f32) -> crate::core::TouchKind {
    if ball_height >= crate::core::TOUCH_HEIGHT_HEAD_MIN
        && ball_height <= crate::core::TOUCH_HEIGHT_HEAD_MAX
        && phase > 0.68
    {
        return crate::core::TouchKind::Head;
    }

    if distance < 3.0 && phase < 0.25 {
        return crate::core::TouchKind::Juggle;
    }

    crate::core::TouchKind::Kick
}

fn compute_zone_drift_target(home: Vec2, zone_radius: f32, phase: f32) -> Vec2 {
    let orbit_radius = zone_radius * (0.35 + 0.45 * (phase * 0.43).sin().abs());
    let direction = Vec2::new(phase.cos(), phase.sin()).normalize_or_zero();
    clamp_to_zone(home, home + direction * orbit_radius, zone_radius)
}

fn clamp_to_zone(home: Vec2, point: Vec2, radius: f32) -> Vec2 {
    let offset = point - home;
    if offset.length() <= radius {
        point
    } else {
        home + offset.normalize_or_zero() * radius
    }
}
