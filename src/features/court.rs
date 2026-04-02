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
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        components::Court,
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(crate::core::COURT_WIDTH, crate::core::COURT_DEPTH),
            ),
        )
    ));

    decor::spawn_courtyard(&mut commands, &asset_server);
    trees::spawn_corner_sakura_trees(&mut commands, &asset_server);
}

fn despawn_court(mut commands: Commands, court_query: Query<Entity, With<components::Court>>) {
    for entity in &court_query {
        commands.entity(entity).despawn();
    }
}
