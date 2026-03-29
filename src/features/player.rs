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
                player_movement.run_if(in_state(crate::core::GameState::InGame)),
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

    let movement = direction.normalize() * crate::core::PLAYER_SPEED * time.delta_secs();
    let max_x = (crate::core::COURT_WIDTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;
    let max_z = (crate::core::COURT_DEPTH * 0.5) - crate::core::PLAYER_COLLIDER_RADIUS;

    for mut transform in &mut player_query {
        transform.translation.x += movement.x;
        transform.translation.z += movement.y;
        transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
        transform.translation.z = transform.translation.z.clamp(-max_z, max_z);
        transform.translation.y = crate::core::PLAYER_Y;
    }
}

fn despawn_players(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in &player_query {
        commands.entity(entity).despawn();
    }
}
