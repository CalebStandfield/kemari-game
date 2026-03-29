mod components;
mod physics;
mod reset;
mod spawn;
mod systems;
mod trajectory;

use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

pub use components::{Ball, BallGroundState, BallVelocity};

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
    player_query: Query<&Transform, With<crate::core::PlayerBody>>,
    mut ball_query: Query<
        (&Transform, &mut BallVelocity, &mut BallGroundState),
        (With<Ball>, Without<crate::core::PlayerBody>),
    >,
) {
    let Ok((ball_transform, mut ball_velocity, mut ball_ground_state)) = ball_query.single_mut()
    else {
        return;
    };
    let ball_height = ball_transform.translation.y;

    for attempt in touch_attempt_reader.read() {
        let Ok(player_transform) = player_query.get(attempt.player) else {
            continue;
        };
        let profile = touch_profile(attempt.kind);

        let delta = Vec2::new(
            ball_transform.translation.x - player_transform.translation.x,
            ball_transform.translation.z - player_transform.translation.z,
        );
        let distance = delta.length();

        let in_radius = distance <= profile.radius;
        let in_height_band = (profile.height_min..=profile.height_max).contains(&ball_height);
        let is_ground_juggle = attempt.kind == crate::core::TouchKind::Juggle
            && ball_ground_state.grounded
            && ball_height <= crate::core::TOUCH_GROUND_JUGGLE_HEIGHT_MAX;
        let is_head_under_ball = attempt.kind != crate::core::TouchKind::Head
            || ball_height > (crate::core::PLAYER_Y + crate::core::PLAYER_HEIGHT * 0.5);

        if !(in_radius && (in_height_band || is_ground_juggle) && is_head_under_ball) {
            whiffed_writer.write(crate::core::BallWhiffedEvent {
                player: attempt.player,
                kind: attempt.kind,
            });
            continue;
        }

        let facing = attempt.facing.normalize_or_zero();
        if facing == Vec2::ZERO {
            whiffed_writer.write(crate::core::BallWhiffedEvent {
                player: attempt.player,
                kind: attempt.kind,
            });
            continue;
        }

        let mut horizontal_velocity = Vec2::new(ball_velocity.linear.x, ball_velocity.linear.z);
        horizontal_velocity *= profile.horizontal_damp;
        if is_ground_juggle {
            // Ground juggle should feel like taking control of a dead ball.
            horizontal_velocity = Vec2::ZERO;
        }
        horizontal_velocity += facing * profile.forward_impulse;
        ball_velocity.linear.x = horizontal_velocity.x;
        ball_velocity.linear.z = horizontal_velocity.y;
        ball_velocity.linear.y =
            (ball_velocity.linear.y * profile.vertical_cushion).max(profile.upward_impulse);

        ball_ground_state.grounded = false;
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
    mut hit_ground_writer: MessageWriter<crate::core::BallHitGroundEvent>,
    mut ball_query: Query<
        (&mut Transform, &mut BallVelocity, &mut BallGroundState),
        (With<Ball>, Without<crate::core::PlayerBody>),
    >,
) {
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    for (mut ball_transform, mut ball_velocity, mut ball_ground_state) in &mut ball_query {
        let was_grounded = ball_ground_state.grounded;

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

        resolve_ball_court_bounds(&mut ball_transform.translation, &mut ball_velocity.linear);
        clamp_ball_horizontal_speed(&mut ball_velocity.linear);

        ball_ground_state.grounded =
            ball_transform.translation.y <= crate::core::BALL_RADIUS + 0.0001;
        if !was_grounded && ball_ground_state.grounded {
            hit_ground_writer.write(crate::core::BallHitGroundEvent);
        }
    }
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
    forward_impulse: f32,
    upward_impulse: f32,
    horizontal_damp: f32,
    vertical_cushion: f32,
}

fn touch_profile(kind: crate::core::TouchKind) -> TouchProfile {
    match kind {
        crate::core::TouchKind::Kick => TouchProfile {
            radius: crate::core::TOUCH_RADIUS_KICK,
            height_min: crate::core::TOUCH_HEIGHT_KICK_MIN,
            height_max: crate::core::TOUCH_HEIGHT_KICK_MAX,
            forward_impulse: crate::core::TOUCH_FORWARD_IMPULSE_KICK,
            upward_impulse: crate::core::TOUCH_UP_IMPULSE_KICK,
            horizontal_damp: 1.0,
            vertical_cushion: 1.0,
        },
        crate::core::TouchKind::Head => TouchProfile {
            radius: crate::core::TOUCH_RADIUS_HEAD,
            height_min: crate::core::TOUCH_HEIGHT_HEAD_MIN,
            height_max: crate::core::TOUCH_HEIGHT_HEAD_MAX,
            forward_impulse: crate::core::TOUCH_FORWARD_IMPULSE_HEAD,
            upward_impulse: crate::core::TOUCH_UP_IMPULSE_HEAD,
            horizontal_damp: 0.9,
            vertical_cushion: crate::core::TOUCH_HEAD_VERTICAL_CUSHION,
        },
        crate::core::TouchKind::Juggle => TouchProfile {
            radius: crate::core::TOUCH_RADIUS_JUGGLE,
            height_min: crate::core::TOUCH_HEIGHT_JUGGLE_MIN,
            height_max: crate::core::TOUCH_HEIGHT_JUGGLE_MAX,
            forward_impulse: crate::core::TOUCH_FORWARD_IMPULSE_JUGGLE,
            upward_impulse: crate::core::TOUCH_UP_IMPULSE_JUGGLE,
            horizontal_damp: crate::core::TOUCH_JUGGLE_HORIZONTAL_DAMP,
            vertical_cushion: 1.0,
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
