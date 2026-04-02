use bevy::prelude::*;

pub fn spawn_courtyard(commands: &mut Commands, asset_server: &AssetServer) {
    let courtyard_scene: Handle<Scene> = asset_server.load(crate::core::COURTYARD_SCENE_PATH);
    let courtyard_position = Vec3::new(
        crate::core::COURTYARD_OFFSET_X,
        crate::core::COURT_Y + crate::core::COURTYARD_OFFSET_Y,
        crate::core::COURTYARD_OFFSET_Z,
    );

    commands.spawn((
        super::components::Court,
        SceneRoot(courtyard_scene),
        Transform::from_translation(courtyard_position)
            .with_scale(Vec3::splat(crate::core::COURTYARD_SCALE)),
    ));
}
