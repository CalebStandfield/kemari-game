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
