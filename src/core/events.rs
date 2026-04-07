use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TouchKind {
    Kick,
    Head,
    Juggle,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerTouchAttemptEvent {
    pub player: Entity,
    pub kind: TouchKind,
    pub facing: Vec2,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct BallTouchedEvent {
    pub player: Entity,
    pub kind: TouchKind,
    pub quality: f32,
    pub ball_height: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct BallWhiffedEvent {
    pub player: Entity,
    pub kind: TouchKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct BallHitGroundEvent;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PassTargetAccuracy {
    CorrectQueueTarget,
    IncorrectQueueTarget,
    NoQueueTarget,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct PassResolutionEvent {
    pub passer: Entity,
    pub receiver: Entity,
    pub expected_target: Option<Entity>,
    pub accuracy: PassTargetAccuracy,
    pub elegance_multiplier: f32,
}

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameplaySet {
    PlayerInput,
    BallResolve,
    Scoring,
    Ritual,
    Ui,
}
