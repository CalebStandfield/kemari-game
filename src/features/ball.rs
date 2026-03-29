mod components;
mod physics;
mod reset;
mod spawn;
mod systems;
mod trajectory;

use bevy::prelude::*;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::core::GameState::InGame), spawn_ball)
            .add_systems(
                Update,
                simulate_ball.run_if(in_state(crate::core::GameState::InGame)),
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
        components::Ball,
        components::BallVelocity::default(),
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

fn despawn_ball(mut commands: Commands, ball_query: Query<Entity, With<components::Ball>>) {
    for entity in &ball_query {
        commands.entity(entity).despawn();
    }
}

fn simulate_ball(
    time: Res<Time>,
    mut ball_query: Query<
        (&mut Transform, &mut components::BallVelocity),
        (With<components::Ball>, Without<crate::features::player::Player>),
    >,
    player_query: Query<&Transform, (With<crate::features::player::Player>, Without<components::Ball>)>,
) {
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }

    for (mut ball_transform, mut ball_velocity) in &mut ball_query {
        resolve_player_ball_collisions(
            &mut ball_transform.translation,
            &mut ball_velocity.linear,
            &player_query,
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

        resolve_ball_court_bounds(&mut ball_transform.translation, &mut ball_velocity.linear);
        clamp_ball_horizontal_speed(&mut ball_velocity.linear);
    }
}

fn resolve_player_ball_collisions(
    ball_position: &mut Vec3,
    ball_velocity: &mut Vec3,
    player_query: &Query<
        &Transform,
        (With<crate::features::player::Player>, Without<components::Ball>),
    >,
) {
    let collision_distance = crate::core::BALL_RADIUS + crate::core::PLAYER_COLLIDER_RADIUS;

    for player_transform in player_query.iter() {
        let separation = Vec2::new(
            ball_position.x - player_transform.translation.x,
            ball_position.z - player_transform.translation.z,
        );
        let distance = separation.length();

        if distance >= collision_distance {
            continue;
        }

        let normal = if distance > f32::EPSILON {
            separation / distance
        } else {
            Vec2::X
        };
        let penetration = collision_distance - distance;
        ball_position.x += normal.x * penetration;
        ball_position.z += normal.y * penetration;

        let mut horizontal_velocity = Vec2::new(ball_velocity.x, ball_velocity.z);
        let velocity_along_normal = horizontal_velocity.dot(normal);
        if velocity_along_normal < 0.0 {
            horizontal_velocity -=
                (1.0 + crate::core::BALL_PLAYER_RESTITUTION) * velocity_along_normal * normal;
        }
        horizontal_velocity += normal * crate::core::BALL_PLAYER_PUSH_SPEED;

        ball_velocity.x = horizontal_velocity.x;
        ball_velocity.z = horizontal_velocity.y;
        ball_velocity.y = ball_velocity.y.max(crate::core::BALL_MIN_BOUNCE_SPEED);
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
