mod bounds;
mod components;
mod decor;
mod spawn;
mod systems;
mod trees;

use bevy::prelude::*;

pub struct CourtPlugin;

impl Plugin for CourtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::core::GameState::InGame), spawn_court)
            .add_systems(OnExit(crate::core::GameState::InGame), despawn_court);
    }
}

fn spawn_court(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        components::Court,
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(crate::core::COURT_WIDTH, crate::core::COURT_DEPTH),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.48, 0.25),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, crate::core::COURT_Y, 0.0),
    ));
}

fn despawn_court(mut commands: Commands, court_query: Query<Entity, With<components::Court>>) {
    for entity in &court_query {
        commands.entity(entity).despawn();
    }
}
