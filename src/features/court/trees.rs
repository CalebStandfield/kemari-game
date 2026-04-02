use bevy::prelude::*;

pub fn spawn_corner_sakura_trees(commands: &mut Commands, asset_server: &AssetServer) {
    let tree_scene: Handle<Scene> = asset_server.load(crate::core::COURT_CORNER_TREE_SCENE_PATH);

    let half_width = crate::core::COURT_WIDTH * 0.5 + crate::core::COURT_CORNER_TREE_PADDING;
    let half_depth = crate::core::COURT_DEPTH * 0.5 + crate::core::COURT_CORNER_TREE_PADDING;
    let scale = Vec3::splat(crate::core::COURT_CORNER_TREE_SCALE);
    let tree_positions = [
        Vec3::new(-half_width, crate::core::COURT_Y, -half_depth),
        Vec3::new(half_width, crate::core::COURT_Y, -half_depth),
        Vec3::new(-half_width, crate::core::COURT_Y, half_depth),
        Vec3::new(half_width, crate::core::COURT_Y, half_depth),
    ];

    for position in tree_positions {
        commands.spawn((
            super::components::Court,
            SceneRoot(tree_scene.clone()),
            Transform::from_translation(position).with_scale(scale),
        ));
    }
}
