use bevy::prelude::*;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Debug, Clone, Copy)]
pub struct BallVelocity {
    pub linear: Vec3,
}

impl Default for BallVelocity {
    fn default() -> Self {
        Self { linear: Vec3::ZERO }
    }
}
