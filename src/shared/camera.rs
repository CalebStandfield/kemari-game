mod follow;
mod shake;
mod zoom;

use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_main_camera);
    }
}

fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 14.0, 14.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        IsDefaultUiCamera,
        MainCamera,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
