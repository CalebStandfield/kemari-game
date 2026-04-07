mod components;
mod physics;
mod reset;
mod spawn;
mod systems;
mod trajectory;

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub use components::{Ball, BallGroundState, BallIncomingPass, BallVelocity};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::core::GameState::InGame), spawn_ball)
            .add_systems(
                Update,
                (resolve_touch_attempts, simulate_ball)
                    .chain()
                    .in_set(crate::core::GameplaySet::BallResolve)
                    .run_if(in_state(crate::core::GameState::InGame)),
            )
            .add_systems(OnExit(crate::core::GameState::InGame), despawn_ball);
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Ball,
        BallVelocity::default(),
        BallGroundState::default(),
        BallIncomingPass::default(),
        Mesh3d(meshes.add(Sphere::new(crate::core::BALL_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.94, 0.94, 0.92),
            metallic: 0.05,
            perceptual_roughness: 0.45,
            ..default()
        })),
        Transform::from_xyz(
            crate::core::BALL_START_X,
            crate::core::BALL_START_Y,
            crate::core::BALL_START_Z,
        ),
    ));
}

fn despawn_ball(mut commands: Commands, ball_query: Query<Entity, With<Ball>>) {
    for entity in &ball_query {
        commands.entity(entity).despawn();
    }
}

fn resolve_touch_attempts(
    mut touch_attempt_reader: MessageReader<crate::core::PlayerTouchAttemptEvent>,
    mut touched_writer: MessageWriter<crate::core::BallTouchedEvent>,
    mut whiffed_writer: MessageWriter<crate::core::BallWhiffedEvent>,
    mut pass_launched_writer: MessageWriter<crate::core::BallPassLaunchedEvent>,
    possession: Res<crate::features::player::BallPossessionState>,
    player_query: Query<&Transform, With<crate::core::PlayerBody>>,
    mut ball_query: Query<
        (
            &Transform,
            &mut BallVelocity,
            &mut BallGroundState,
            &mut BallIncomingPass,
        ),
        (With<Ball>, Without<crate::core::PlayerBody>),
    >,
) {
    let Ok((ball_transform, mut ball_velocity, mut ball_ground_state, mut incoming_pass)) =
        ball_query.single_mut()
    else {
        return;
    };
    let ball_height = ball_transform.translation.y;

    for attempt in touch_attempt_reader.read() {
        let Ok(player_transform) = player_query.get(attempt.player) else {
            continue;
        };

        let profile = touch_profile(attempt.kind);
        let ball_to_player = Vec2::new(
            ball_transform.translation.x - player_transform.translation.x,
            ball_transform.translation.z - player_transform.translation.z,
        );
        let distance = ball_to_player.length();
        let has_ball_control = possession.holder == Some(attempt.player);

        if has_ball_control {
            let target = attempt.target.filter(|target| *target != attempt.player);

            match attempt.kind {
                crate::core::TouchKind::Kick | crate::core::TouchKind::Head => {
                    let Some(target_entity) = target else {
                        whiffed_writer.write(crate::core::BallWhiffedEvent {
                            player: attempt.player,
                            kind: attempt.kind,
                        });
                        continue;
                    };

                    if attempt.kind == crate::core::TouchKind::Head
                        && ball_height < crate::core::TOUCH_HEIGHT_HEAD_MIN
                    {
                        whiffed_writer.write(crate::core::BallWhiffedEvent {
                            player: attempt.player,
                            kind: attempt.kind,
                        });
                        continue;
                    }

                    let Ok(target_transform) = player_query.get(target_entity) else {
                        whiffed_writer.write(crate::core::BallWhiffedEvent {
                            player: attempt.player,
                            kind: attempt.kind,
                        });
                        continue;
                    };

                    launch_targeted_pass(
                        attempt.player,
                        target_entity,
                        attempt.kind,
                        attempt.facing,
                        target_transform,
                        ball_transform,
                        &mut ball_velocity,
                        &mut ball_ground_state,
                        &mut incoming_pass,
                        &mut pass_launched_writer,
                    );

                    let quality =
                        calculate_touch_quality(distance, profile.radius, ball_height, profile);
                    touched_writer.write(crate::core::BallTouchedEvent {
                        player: attempt.player,
                        kind: attempt.kind,
                        quality,
                        ball_height,
                    });
                }
                crate::core::TouchKind::Juggle => {
                    if let Some(target_entity) = target {
                        let Ok(target_transform) = player_query.get(target_entity) else {
                            whiffed_writer.write(crate::core::BallWhiffedEvent {
                                player: attempt.player,
                                kind: attempt.kind,
                            });
                            continue;
                        };

                        launch_targeted_pass(
                            attempt.player,
                            target_entity,
                            attempt.kind,
                            attempt.facing,
                            target_transform,
                            ball_transform,
                            &mut ball_velocity,
                            &mut ball_ground_state,
                            &mut incoming_pass,
                            &mut pass_launched_writer,
                        );

                        let quality =
                            calculate_touch_quality(distance, profile.radius, ball_height, profile);
                        touched_writer.write(crate::core::BallTouchedEvent {
                            player: attempt.player,
                            kind: attempt.kind,
                            quality,
                            ball_height,
                        });
                        continue;
                    }

                    let in_control_range = distance <= crate::core::BALL_CONTROL_RADIUS
                        && ball_height <= crate::core::BALL_CONTROL_HEIGHT_MAX;
                    if !in_control_range {
                        whiffed_writer.write(crate::core::BallWhiffedEvent {
                            player: attempt.player,
                            kind: attempt.kind,
                        });
                        continue;
                    }

                    let mut horizontal = Vec2::new(ball_velocity.linear.x, ball_velocity.linear.z);
                    horizontal *= crate::core::TOUCH_JUGGLE_HORIZONTAL_DAMP;
                    ball_velocity.linear.x = horizontal.x;
                    ball_velocity.linear.z = horizontal.y;
                    ball_velocity.linear.y =
                        (ball_velocity.linear.y * 0.25).max(crate::core::TOUCH_UP_IMPULSE_JUGGLE);
                    ball_ground_state.grounded = false;
                    clear_incoming_pass(&mut incoming_pass);

                    let quality =
                        calculate_touch_quality(distance, profile.radius, ball_height, profile);
                    touched_writer.write(crate::core::BallTouchedEvent {
                        player: attempt.player,
                        kind: attempt.kind,
                        quality,
                        ball_height,
                    });
                }
            }

            continue;
        }

        if attempt.kind != crate::core::TouchKind::Juggle {
            whiffed_writer.write(crate::core::BallWhiffedEvent {
                player: attempt.player,
                kind: attempt.kind,
            });
            continue;
        }

        let in_radius = distance <= profile.radius;
        let in_height_band = ball_height <= crate::core::TOUCH_HEIGHT_JUGGLE_MAX
            || (ball_ground_state.grounded
                && ball_height <= crate::core::TOUCH_GROUND_JUGGLE_HEIGHT_MAX);

        if !(in_radius && in_height_band) {
            whiffed_writer.write(crate::core::BallWhiffedEvent {
                player: attempt.player,
                kind: attempt.kind,
            });
            continue;
        }

        ball_velocity.linear.x = 0.0;
        ball_velocity.linear.z = 0.0;
        ball_velocity.linear.y = crate::core::TOUCH_UP_IMPULSE_JUGGLE * 0.84;
        ball_ground_state.grounded = false;
        clear_incoming_pass(&mut incoming_pass);

        let quality = calculate_touch_quality(distance, profile.radius, ball_height, profile);
        touched_writer.write(crate::core::BallTouchedEvent {
            player: attempt.player,
            kind: attempt.kind,
            quality,
            ball_height,
        });
    }
}

fn simulate_ball(
    time: Res<Time>,
    possession: Res<crate::features::player::BallPossessionState>,
    mut hit_ground_writer: MessageWriter<crate::core::BallHitGroundEvent>,
    mut touched_writer: MessageWriter<crate::core::BallTouchedEvent>,
    mut ball_query: Query<
        (
            &mut Transform,
            &mut BallVelocity,
            &mut BallGroundState,
            &mut BallIncomingPass,
        ),
        (With<Ball>, Without<crate::core::PlayerBody>),
    >,
    player_query: Query<
        (
            Entity,
            &Transform,
            Option<&crate::features::player::ControlledPlayer>,
            Option<&crate::features::player::PlayerFacing>,
        ),
        (With<crate::core::PlayerBody>, Without<Ball>),
    >,
    player_contact_query: Query<
        (Entity, &Transform),
        (With<crate::core::PlayerBody>, Without<Ball>),
    >,
    mut player_prev_positions: Local<HashMap<Entity, Vec2>>,
) {
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    for (mut ball_transform, mut ball_velocity, mut ball_ground_state, mut incoming_pass) in
        &mut ball_query
    {
        let was_grounded = ball_ground_state.grounded;

        apply_holder_control_bias(
            &possession,
            &player_query,
            &mut ball_transform.translation,
            &mut ball_velocity.linear,
            delta_seconds,
        );

        apply_receiver_assist(
            &mut incoming_pass,
            &player_query,
            &mut ball_transform.translation,
            &mut ball_velocity.linear,
            delta_seconds,
        );

        ball_velocity.linear.y -= crate::core::BALL_GRAVITY * delta_seconds;
        ball_transform.translation += ball_velocity.linear * delta_seconds;

        if ball_transform.translation.y < crate::core::BALL_RADIUS {
            ball_transform.translation.y = crate::core::BALL_RADIUS;

            if ball_velocity.linear.y < 0.0 {
                ball_velocity.linear.y = -ball_velocity.linear.y * crate::core::BALL_GROUND_BOUNCE;
                if ball_velocity.linear.y.abs() < crate::core::BALL_MIN_BOUNCE_SPEED {
                    ball_velocity.linear.y = 0.0;
                }
            }

            let mut horizontal_velocity = Vec2::new(ball_velocity.linear.x, ball_velocity.linear.z);
            let horizontal_speed = horizontal_velocity.length();
            if horizontal_speed > 0.0 {
                let deceleration =
                    (crate::core::BALL_GROUND_FRICTION * delta_seconds).min(horizontal_speed);
                horizontal_velocity =
                    horizontal_velocity.normalize_or_zero() * (horizontal_speed - deceleration);
                ball_velocity.linear.x = horizontal_velocity.x;
                ball_velocity.linear.z = horizontal_velocity.y;
            }
        } else {
            let drag_factor = (1.0 - (crate::core::BALL_AIR_DRAG * delta_seconds)).clamp(0.0, 1.0);
            ball_velocity.linear.x *= drag_factor;
            ball_velocity.linear.z *= drag_factor;
        }

        resolve_player_foot_contacts(
            &mut ball_transform.translation,
            &mut ball_velocity.linear,
            ball_ground_state.grounded,
            delta_seconds,
            &player_contact_query,
            &mut player_prev_positions,
        );
        resolve_ball_court_bounds(&mut ball_transform.translation, &mut ball_velocity.linear);
        clamp_ball_horizontal_speed(&mut ball_velocity.linear);

        ball_ground_state.grounded =
            ball_transform.translation.y <= crate::core::BALL_RADIUS + 0.0001;

        if let Some(receiver) = incoming_pass.receiver {
            if possession.holder.is_none() {
                try_auto_receive_for_npc(
                    receiver,
                    incoming_pass.kind.unwrap_or(crate::core::TouchKind::Juggle),
                    &player_query,
                    &mut ball_transform.translation,
                    &mut ball_velocity.linear,
                    ball_ground_state.grounded,
                    &mut touched_writer,
                    &mut incoming_pass,
                );
            }
        }

        if !was_grounded && ball_ground_state.grounded {
            hit_ground_writer.write(crate::core::BallHitGroundEvent);
            clear_incoming_pass(&mut incoming_pass);
        }
    }
}

fn apply_holder_control_bias(
    possession: &crate::features::player::BallPossessionState,
    player_query: &Query<
        (
            Entity,
            &Transform,
            Option<&crate::features::player::ControlledPlayer>,
            Option<&crate::features::player::PlayerFacing>,
        ),
        (With<crate::core::PlayerBody>, Without<Ball>),
    >,
    ball_position: &mut Vec3,
    ball_velocity: &mut Vec3,
    delta_seconds: f32,
) {
    let Some(holder) = possession.holder else {
        return;
    };

    let Ok((_entity, holder_transform, _controlled, holder_facing)) = player_query.get(holder)
    else {
        return;
    };

    if ball_position.y > crate::core::BALL_CONTROL_HEIGHT_MAX {
        return;
    }

    let holder_position = Vec2::new(
        holder_transform.translation.x,
        holder_transform.translation.z,
    );
    let facing = holder_facing
        .map(|facing| facing.0.normalize_or_zero())
        .filter(|facing| *facing != Vec2::ZERO)
        .unwrap_or(Vec2::Y);

    let control_point = holder_position + facing * crate::core::BALL_CONTROL_FORWARD_OFFSET;
    let ball_xz = Vec2::new(ball_position.x, ball_position.z);
    let to_control = control_point - ball_xz;

    if to_control.length() > crate::core::BALL_CONTROL_RADIUS {
        return;
    }

    ball_velocity.x += to_control.x * crate::core::BALL_CONTROL_MAGNET_STRENGTH * delta_seconds;
    ball_velocity.z += to_control.y * crate::core::BALL_CONTROL_MAGNET_STRENGTH * delta_seconds;
    ball_velocity.x *= crate::core::BALL_CONTROL_MAGNET_DAMP;
    ball_velocity.z *= crate::core::BALL_CONTROL_MAGNET_DAMP;
}

fn apply_receiver_assist(
    incoming_pass: &mut BallIncomingPass,
    player_query: &Query<
        (
            Entity,
            &Transform,
            Option<&crate::features::player::ControlledPlayer>,
            Option<&crate::features::player::PlayerFacing>,
        ),
        (With<crate::core::PlayerBody>, Without<Ball>),
    >,
    ball_position: &mut Vec3,
    ball_velocity: &mut Vec3,
    delta_seconds: f32,
) {
    let Some(receiver) = incoming_pass.receiver else {
        return;
    };

    let Ok((_entity, receiver_transform, _controlled, _facing)) = player_query.get(receiver) else {
        clear_incoming_pass(incoming_pass);
        return;
    };

    let receiver_position = Vec2::new(
        receiver_transform.translation.x,
        receiver_transform.translation.z,
    );
    let ball_xz = Vec2::new(ball_position.x, ball_position.z);
    let to_receiver = receiver_position - ball_xz;

    if to_receiver.length_squared() <= f32::EPSILON {
        return;
    }

    let steer =
        to_receiver.normalize_or_zero() * crate::core::BALL_PASS_RECEIVE_STEER * delta_seconds;
    ball_velocity.x += steer.x;
    ball_velocity.z += steer.y;
}

fn try_auto_receive_for_npc(
    receiver: Entity,
    receive_kind: crate::core::TouchKind,
    player_query: &Query<
        (
            Entity,
            &Transform,
            Option<&crate::features::player::ControlledPlayer>,
            Option<&crate::features::player::PlayerFacing>,
        ),
        (With<crate::core::PlayerBody>, Without<Ball>),
    >,
    ball_position: &mut Vec3,
    ball_velocity: &mut Vec3,
    ball_grounded: bool,
    touched_writer: &mut MessageWriter<crate::core::BallTouchedEvent>,
    incoming_pass: &mut BallIncomingPass,
) {
    let Ok((_entity, receiver_transform, receiver_controlled, _)) = player_query.get(receiver)
    else {
        clear_incoming_pass(incoming_pass);
        return;
    };

    if receiver_controlled.is_some() {
        return;
    }

    let receiver_position = Vec2::new(
        receiver_transform.translation.x,
        receiver_transform.translation.z,
    );
    let ball_xz = Vec2::new(ball_position.x, ball_position.z);
    let distance = (receiver_position - ball_xz).length();
    let height_ok = (crate::core::BALL_PASS_RECEIVE_HEIGHT_MIN
        ..=crate::core::BALL_PASS_RECEIVE_HEIGHT_MAX)
        .contains(&ball_position.y);
    let descending_enough = ball_velocity.y <= 1.0 || ball_grounded;

    if distance > crate::core::BALL_PASS_RECEIVE_SNAP_RADIUS || !height_ok || !descending_enough {
        return;
    }

    let approach = (receiver_position - ball_xz).normalize_or_zero();
    ball_position.x =
        receiver_position.x - approach.x * crate::core::BALL_PASS_RECEIVE_POINT_OFFSET;
    ball_position.z =
        receiver_position.y - approach.y * crate::core::BALL_PASS_RECEIVE_POINT_OFFSET;
    ball_position.y = crate::core::BALL_RADIUS;
    *ball_velocity = Vec3::ZERO;

    let quality = (1.0 - (distance / crate::core::BALL_PASS_RECEIVE_SNAP_RADIUS)).clamp(0.0, 1.0);
    touched_writer.write(crate::core::BallTouchedEvent {
        player: receiver,
        kind: receive_kind,
        quality,
        ball_height: ball_position.y,
    });

    clear_incoming_pass(incoming_pass);
}

fn launch_targeted_pass(
    passer: Entity,
    receiver: Entity,
    kind: crate::core::TouchKind,
    facing: Vec2,
    receiver_transform: &Transform,
    ball_transform: &Transform,
    ball_velocity: &mut BallVelocity,
    ball_ground_state: &mut BallGroundState,
    incoming_pass: &mut BallIncomingPass,
    pass_launched_writer: &mut MessageWriter<crate::core::BallPassLaunchedEvent>,
) {
    let source = Vec2::new(ball_transform.translation.x, ball_transform.translation.z);
    let facing = facing.normalize_or_zero();
    let facing_bias = if facing == Vec2::ZERO {
        Vec2::ZERO
    } else {
        facing * crate::core::BALL_PASS_RECEIVE_POINT_OFFSET
    };
    let target = Vec2::new(
        receiver_transform.translation.x,
        receiver_transform.translation.z,
    ) + facing_bias;
    let to_target = target - source;
    let horizontal_distance = to_target.length().max(0.001);

    let horizontal_speed = pass_horizontal_speed(kind).max(0.001);
    let travel_time = (horizontal_distance / horizontal_speed).clamp(
        crate::core::BALL_PASS_MIN_TIME,
        crate::core::BALL_PASS_MAX_TIME,
    );

    let target_height = pass_target_height(kind);
    let vertical_velocity = (target_height - ball_transform.translation.y
        + 0.5 * crate::core::BALL_GRAVITY * travel_time * travel_time)
        / travel_time;

    let horizontal_velocity = to_target / travel_time;
    ball_velocity.linear.x = horizontal_velocity.x;
    ball_velocity.linear.z = horizontal_velocity.y;
    ball_velocity.linear.y = vertical_velocity;
    ball_ground_state.grounded = false;

    incoming_pass.passer = Some(passer);
    incoming_pass.receiver = Some(receiver);
    incoming_pass.kind = Some(kind);

    pass_launched_writer.write(crate::core::BallPassLaunchedEvent {
        passer,
        receiver,
        kind,
    });
}

fn pass_horizontal_speed(kind: crate::core::TouchKind) -> f32 {
    match kind {
        crate::core::TouchKind::Kick => crate::core::BALL_PASS_SPEED_KICK,
        crate::core::TouchKind::Head => crate::core::BALL_PASS_SPEED_HEAD,
        crate::core::TouchKind::Juggle => crate::core::BALL_PASS_SPEED_JUGGLE,
    }
}

fn pass_target_height(kind: crate::core::TouchKind) -> f32 {
    match kind {
        crate::core::TouchKind::Kick => crate::core::BALL_PASS_TARGET_HEIGHT_KICK,
        crate::core::TouchKind::Head => crate::core::BALL_PASS_TARGET_HEIGHT_HEAD,
        crate::core::TouchKind::Juggle => crate::core::BALL_PASS_TARGET_HEIGHT_JUGGLE,
    }
}

fn clear_incoming_pass(incoming_pass: &mut BallIncomingPass) {
    incoming_pass.passer = None;
    incoming_pass.receiver = None;
    incoming_pass.kind = None;
}

fn resolve_player_foot_contacts(
    ball_position: &mut Vec3,
    ball_velocity: &mut Vec3,
    ball_grounded: bool,
    delta_seconds: f32,
    player_query: &Query<(Entity, &Transform), (With<crate::core::PlayerBody>, Without<Ball>)>,
    player_prev_positions: &mut HashMap<Entity, Vec2>,
) {
    let mut seen_players = HashSet::new();
    let contact_distance = crate::core::BALL_RADIUS + crate::core::PLAYER_COLLIDER_RADIUS;
    let contact_distance_sq = contact_distance * contact_distance;
    let can_collide_at_feet = ball_position.y <= crate::core::BALL_PLAYER_CONTACT_HEIGHT_MAX;

    for (player_entity, player_transform) in player_query.iter() {
        seen_players.insert(player_entity);

        let player_position = Vec2::new(
            player_transform.translation.x,
            player_transform.translation.z,
        );
        let previous_position = player_prev_positions
            .insert(player_entity, player_position)
            .unwrap_or(player_position);
        let player_displacement = player_position - previous_position;

        if !can_collide_at_feet {
            continue;
        }

        let separation = Vec2::new(
            ball_position.x - player_position.x,
            ball_position.z - player_position.y,
        );
        let distance_sq = separation.length_squared();
        if distance_sq >= contact_distance_sq {
            continue;
        }

        let distance = distance_sq.sqrt();
        let normal = if distance > f32::EPSILON {
            separation / distance
        } else if player_displacement.length_squared() > f32::EPSILON {
            player_displacement.normalize_or_zero()
        } else {
            Vec2::X
        };

        let penetration = contact_distance - distance;
        ball_position.x += normal.x * penetration;
        ball_position.z += normal.y * penetration;

        let mut horizontal_velocity = Vec2::new(ball_velocity.x, ball_velocity.z);
        let toward_player_speed = horizontal_velocity.dot(normal);
        if toward_player_speed < 0.0 {
            horizontal_velocity -= normal * toward_player_speed;
        }
        horizontal_velocity *= crate::core::BALL_PLAYER_CONTACT_DAMP;

        let displacement_len = player_displacement.length();
        if displacement_len > f32::EPSILON {
            let move_direction = player_displacement / displacement_len;
            let move_speed = displacement_len / delta_seconds.max(0.0001);
            let moving_into_ball = move_direction.dot(normal).max(0.0);
            let push_amount = crate::core::BALL_PLAYER_WALK_PUSH * move_speed * moving_into_ball;
            horizontal_velocity += move_direction * push_amount;
        } else if ball_grounded
            && horizontal_velocity.length_squared()
                < crate::core::BALL_PLAYER_REST_SPEED * crate::core::BALL_PLAYER_REST_SPEED
        {
            horizontal_velocity = Vec2::ZERO;
        }

        ball_velocity.x = horizontal_velocity.x;
        ball_velocity.z = horizontal_velocity.y;
    }

    player_prev_positions.retain(|entity, _| seen_players.contains(entity));
}

fn resolve_ball_court_bounds(ball_position: &mut Vec3, ball_velocity: &mut Vec3) {
    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::BALL_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::BALL_RADIUS;

    if ball_position.x > max_x {
        ball_position.x = max_x;
        if ball_velocity.x > 0.0 {
            ball_velocity.x = -ball_velocity.x * crate::core::BALL_WALL_BOUNCE;
        }
    } else if ball_position.x < -max_x {
        ball_position.x = -max_x;
        if ball_velocity.x < 0.0 {
            ball_velocity.x = -ball_velocity.x * crate::core::BALL_WALL_BOUNCE;
        }
    }

    if ball_position.z > max_z {
        ball_position.z = max_z;
        if ball_velocity.z > 0.0 {
            ball_velocity.z = -ball_velocity.z * crate::core::BALL_WALL_BOUNCE;
        }
    } else if ball_position.z < -max_z {
        ball_position.z = -max_z;
        if ball_velocity.z < 0.0 {
            ball_velocity.z = -ball_velocity.z * crate::core::BALL_WALL_BOUNCE;
        }
    }
}

fn clamp_ball_horizontal_speed(ball_velocity: &mut Vec3) {
    let mut horizontal_velocity = Vec2::new(ball_velocity.x, ball_velocity.z);
    let horizontal_speed = horizontal_velocity.length();

    if horizontal_speed > crate::core::BALL_MAX_HORIZONTAL_SPEED {
        horizontal_velocity =
            horizontal_velocity.normalize() * crate::core::BALL_MAX_HORIZONTAL_SPEED;
        ball_velocity.x = horizontal_velocity.x;
        ball_velocity.z = horizontal_velocity.y;
    }
}

#[derive(Debug, Clone, Copy)]
struct TouchProfile {
    radius: f32,
    height_min: f32,
    height_max: f32,
}

fn touch_profile(kind: crate::core::TouchKind) -> TouchProfile {
    match kind {
        crate::core::TouchKind::Kick => TouchProfile {
            radius: crate::core::TOUCH_RADIUS_KICK,
            height_min: crate::core::TOUCH_HEIGHT_KICK_MIN,
            height_max: crate::core::TOUCH_HEIGHT_KICK_MAX,
        },
        crate::core::TouchKind::Head => TouchProfile {
            radius: crate::core::TOUCH_RADIUS_HEAD,
            height_min: crate::core::TOUCH_HEIGHT_HEAD_MIN,
            height_max: crate::core::TOUCH_HEIGHT_HEAD_MAX,
        },
        crate::core::TouchKind::Juggle => TouchProfile {
            radius: crate::core::TOUCH_RADIUS_JUGGLE,
            height_min: crate::core::TOUCH_HEIGHT_JUGGLE_MIN,
            height_max: crate::core::TOUCH_HEIGHT_JUGGLE_MAX,
        },
    }
}

fn calculate_touch_quality(
    distance: f32,
    radius: f32,
    ball_height: f32,
    profile: TouchProfile,
) -> f32 {
    let distance_quality = 1.0 - (distance / radius.max(0.001)).clamp(0.0, 1.0);
    let middle_height = (profile.height_min + profile.height_max) * 0.5;
    let half_band = ((profile.height_max - profile.height_min) * 0.5).max(0.001);
    let height_quality = 1.0 - ((ball_height - middle_height).abs() / half_band).clamp(0.0, 1.0);
    (distance_quality * 0.6 + height_quality * 0.4).clamp(0.0, 1.0)
}
