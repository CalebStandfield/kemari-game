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

pub use components::{ControlledPlayer, Player, PlayerDisplayName};
pub use pass_queue::PlayerPassRequestQueue;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<pass_queue::PlayerPassRequestQueue>()
            .init_resource::<pass_queue::BallPossessionState>()
            .init_resource::<pass_queue::PassQueueDebugState>()
            .add_systems(OnEnter(crate::core::GameState::InGame), spawn_players)
            .add_systems(OnExit(crate::core::GameState::InGame), despawn_players)
            .add_systems(
                OnExit(crate::core::GameState::InGame),
                pass_queue::reset_pass_queue_state,
            )
            .add_systems(
                Update,
                (
                    tick_touch_cooldowns,
                    player_movement,
                    update_controlled_player_call_state,
                    pass_queue::tick_npc_rejoin_cooldowns,
                    pass_queue::sync_queue_from_call_state,
                    pass_queue::prune_invalid_queue_members,
                    emit_touch_attempts,
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
    let ring_radius = if player_count == 1 { 0.0 } else { 3.8 };
    let ring_center_x = crate::core::BALL_START_X;
    let ring_center_z = crate::core::BALL_START_Z;
    let player_scene: Handle<Scene> = asset_server.load(crate::core::PLAYER_SCENE_PATH);

    for player_index in 0..player_count {
        let angle = (player_index as f32 / player_count as f32) * TAU;
        let x = ring_center_x + ring_radius * angle.cos();
        let z = ring_center_z + ring_radius * angle.sin();
        let is_controlled = player_index == 0;
        let display_name = format!("Player {}", player_index + 1);

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
            components::PlayerCallForBall::default(),
            components::PlayerFacing::default(),
            components::PlayerTouchCooldowns::default(),
            transform,
        ));

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
        callout::spawn_player_callout(&mut commands, player_entity, &display_name);
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (&mut Transform, &mut components::PlayerFacing),
        With<ControlledPlayer>,
    >,
) {
    let mut direction = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction == Vec2::ZERO {
        return;
    }

    let delta_seconds = time.delta_secs();
    let normalized_direction = direction.normalize();
    let movement = normalized_direction * crate::core::PLAYER_SPEED * delta_seconds;
    let facing_direction = Vec3::new(normalized_direction.x, 0.0, normalized_direction.y);
    let target_rotation = Transform::IDENTITY
        .looking_to(facing_direction, Vec3::Y)
        .rotation;
    let turn_lerp = (crate::core::PLAYER_TURN_SPEED * delta_seconds).clamp(0.0, 1.0);
    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;

    for (mut transform, mut facing) in &mut player_query {
        facing.0 = normalized_direction;
        transform.rotation = transform.rotation.slerp(target_rotation, turn_lerp);
        transform.translation.x += movement.x;
        transform.translation.z += movement.y;
        transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
        transform.translation.z = transform.translation.z.clamp(-max_z, max_z);
        transform.translation.y = crate::core::PLAYER_Y;
    }
}

fn tick_touch_cooldowns(
    time: Res<Time>,
    mut player_query: Query<&mut components::PlayerTouchCooldowns, With<ControlledPlayer>>,
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

fn update_controlled_player_call_state(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    possession: Res<pass_queue::BallPossessionState>,
    mut player_query: Query<(Entity, &mut components::PlayerCallForBall), With<ControlledPlayer>>,
) {
    let wants_to_call = keyboard_input.pressed(KeyCode::KeyL);

    for (player, mut call_for_ball) in &mut player_query {
        let has_ball_control = possession.holder == Some(player);
        call_for_ball.active = wants_to_call && !has_ball_control;
    }
}

fn emit_touch_attempts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut touch_writer: MessageWriter<crate::core::PlayerTouchAttemptEvent>,
    mut player_query: Query<
        (
            Entity,
            &components::PlayerFacing,
            &mut components::PlayerTouchCooldowns,
        ),
        With<ControlledPlayer>,
    >,
) {
    let Ok((player_entity, facing, mut cooldowns)) = player_query.single_mut() else {
        return;
    };

    let facing = facing.0.normalize_or_zero();
    if facing == Vec2::ZERO {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::KeyK) && cooldowns.kick <= 0.0 {
        cooldowns.kick = crate::core::TOUCH_COOLDOWN_KICK;
        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: player_entity,
            kind: crate::core::TouchKind::Kick,
            facing,
        });
    }

    if keyboard_input.just_pressed(KeyCode::KeyH) && cooldowns.head <= 0.0 {
        cooldowns.head = crate::core::TOUCH_COOLDOWN_HEAD;
        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: player_entity,
            kind: crate::core::TouchKind::Head,
            facing,
        });
    }

    if keyboard_input.just_pressed(KeyCode::KeyJ) && cooldowns.juggle <= 0.0 {
        cooldowns.juggle = crate::core::TOUCH_COOLDOWN_JUGGLE;
        touch_writer.write(crate::core::PlayerTouchAttemptEvent {
            player: player_entity,
            kind: crate::core::TouchKind::Juggle,
            facing,
        });
    }
}

fn resolve_player_collisions(mut player_query: Query<&mut Transform, With<Player>>) {
    let min_distance = crate::core::PLAYER_COLLIDER_RADIUS * 2.0;
    let min_distance_squared = min_distance * min_distance;
    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;

    // Multiple passes reduce residual overlap in small groups of touching players.
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
