use bevy::prelude::*;

#[derive(Component)]
pub struct Ball;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BallActionPhase {
    Free,
    TravelingToTarget,
    Controlled,
    PreparingTouch,
    JugglingRecovery,
    Passing,
    Dropped,
    Resetting,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BallActionState {
    pub phase: BallActionPhase,
    pub controller: Option<Entity>,
    pub intended_receiver: Option<Entity>,
}

impl Default for BallActionState {
    fn default() -> Self {
        Self {
            phase: BallActionPhase::Free,
            controller: None,
            intended_receiver: None,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BallVelocity {
    pub linear: Vec3,
}

impl Default for BallVelocity {
    fn default() -> Self {
        Self { linear: Vec3::ZERO }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BallGroundState {
    pub grounded: bool,
}

impl Default for BallGroundState {
    fn default() -> Self {
        Self { grounded: true }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BallGroundedTimeout {
    pub elapsed: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BallIncomingPass {
    pub passer: Option<Entity>,
    pub receiver: Option<Entity>,
    pub kind: Option<crate::core::TouchKind>,
}
