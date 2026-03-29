mod animation;
mod components;
mod kick;
mod movement;
mod spawn;
mod systems;

use bevy::prelude::*;
use std::f32::consts::TAU;

pub use components::{ControlledPlayer, Player};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::core::GameState::InGame), spawn_players)
            .add_systems(OnExit(crate::core::GameState::InGame), despawn_players)
            .add_systems(
                Update,
                (player_movement, kick_ball, resolve_player_collisions)
                    .chain()
                    .run_if(in_state(crate::core::GameState::InGame)),
            );
    }
}

fn spawn_players(
    mut commands: Commands,
    session_config: Res<crate::core::SessionConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_count = session_config.player_count.clamp(1, 8);
    let ring_radius = if player_count == 1 { 0.0 } else { 3.8 };
    let ring_center_x = crate::core::BALL_START_X;
    let ring_center_z = crate::core::BALL_START_Z;

    for player_index in 0..player_count {
        let angle = (player_index as f32 / player_count as f32) * TAU;
        let x = ring_center_x + ring_radius * angle.cos();
        let z = ring_center_z + ring_radius * angle.sin();
        let is_controlled = player_index == 0;

        let mut transform = Transform::from_xyz(x, crate::core::PLAYER_Y, z);
        if player_count > 1 {
            transform.look_at(
                Vec3::new(ring_center_x, crate::core::PLAYER_Y, ring_center_z),
                Vec3::Y,
            );
        }

        let mut entity_commands = commands.spawn((
            Player,
            Mesh3d(meshes.add(Cuboid::new(
                crate::core::PLAYER_WIDTH,
                crate::core::PLAYER_HEIGHT,
                crate::core::PLAYER_DEPTH,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: if is_controlled {
                    Color::srgb(0.90, 0.85, 0.35)
                } else {
                    Color::srgb(0.64, 0.74, 0.80)
                },
                ..default()
            })),
            transform,
        ));

        if is_controlled {
            entity_commands.insert(ControlledPlayer);
        }
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<ControlledPlayer>>,
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

    let normalized_direction = direction.normalize();
    let movement = normalized_direction * crate::core::PLAYER_SPEED * time.delta_secs();
    let facing_direction = Vec3::new(normalized_direction.x, 0.0, normalized_direction.y);
    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;

    for mut transform in &mut player_query {
        transform.look_to(facing_direction, Vec3::Y);
        transform.translation.x += movement.x;
        transform.translation.z += movement.y;
        transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
        transform.translation.z = transform.translation.z.clamp(-max_z, max_z);
        transform.translation.y = crate::core::PLAYER_Y;
    }
}

fn kick_ball(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, (With<ControlledPlayer>, Without<crate::features::ball::Ball>)>,
    mut ball_query: Query<
        (&Transform, &mut crate::features::ball::BallVelocity),
        (With<crate::features::ball::Ball>, Without<ControlledPlayer>),
    >,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok((ball_transform, mut ball_velocity)) = ball_query.single_mut() else {
        return;
    };

    let ball_offset = Vec2::new(
        ball_transform.translation.x - player_transform.translation.x,
        ball_transform.translation.z - player_transform.translation.z,
    );
    let kick_reach = crate::core::PLAYER_COLLIDER_RADIUS
        + crate::core::BALL_RADIUS
        + crate::core::PLAYER_KICK_RANGE;
    if ball_offset.length_squared() > kick_reach * kick_reach {
        return;
    }

    let mut facing_direction = player_transform.rotation * Vec3::NEG_Z;
    facing_direction.y = 0.0;
    let facing_direction = facing_direction.normalize_or_zero();
    if facing_direction == Vec3::ZERO {
        return;
    }

    ball_velocity.linear.x = facing_direction.x * crate::core::PLAYER_KICK_SPEED;
    ball_velocity.linear.z = facing_direction.z * crate::core::PLAYER_KICK_SPEED;
    ball_velocity.linear.y = ball_velocity.linear.y.max(crate::core::PLAYER_KICK_LIFT);
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

fn despawn_players(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in &player_query {
        commands.entity(entity).despawn();
    }
}
