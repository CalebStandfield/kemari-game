use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Component, Debug, Clone)]
pub struct PlayerDisplayName(pub String);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerCallForBall {
    pub active: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerFacing(pub Vec2);

impl Default for PlayerFacing {
    fn default() -> Self {
        Self(Vec2::new(0.0, -1.0))
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerTouchCooldowns {
    pub kick: f32,
    pub head: f32,
    pub juggle: f32,
}

impl Default for PlayerTouchCooldowns {
    fn default() -> Self {
        Self {
            kick: 0.0,
            head: 0.0,
            juggle: 0.0,
        }
    }
}
