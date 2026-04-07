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

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerSlot {
    pub index: usize,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerHomePosition(pub Vec2);

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerZoneRadius(pub f32);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerDesiredMove(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum NpcBehaviorState {
    Idle,
    CallingForPass,
    PreparingToReceive,
    ControllingBall,
    Passing,
    RecoveringToHome,
}

#[derive(Component, Debug, Clone)]
pub struct NpcBehavior {
    pub state: NpcBehaviorState,
    pub drift_target: Vec2,
    pub drift_timer: Timer,
    pub call_decision_timer: Timer,
    pub pass_timer: Timer,
    pub last_pass_target: Option<Entity>,
}

impl NpcBehavior {
    pub fn new(home: Vec2, phase: f32) -> Self {
        Self {
            state: NpcBehaviorState::Idle,
            drift_target: home,
            drift_timer: Timer::from_seconds(
                drift_interval_from_phase(phase),
                TimerMode::Repeating,
            ),
            call_decision_timer: Timer::from_seconds(
                call_interval_from_phase(phase),
                TimerMode::Repeating,
            ),
            pass_timer: Timer::from_seconds(
                pass_hesitation_from_phase(phase),
                TimerMode::Repeating,
            ),
            last_pass_target: None,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SelectedPassTarget {
    pub slot_index: usize,
    pub entity: Option<Entity>,
}

pub fn drift_interval_from_phase(phase: f32) -> f32 {
    lerp(
        crate::core::NPC_IDLE_DRIFT_INTERVAL_MIN,
        crate::core::NPC_IDLE_DRIFT_INTERVAL_MAX,
        phase,
    )
}

pub fn call_interval_from_phase(phase: f32) -> f32 {
    lerp(
        crate::core::NPC_CALL_DECISION_INTERVAL_MIN,
        crate::core::NPC_CALL_DECISION_INTERVAL_MAX,
        phase,
    )
}

pub fn pass_hesitation_from_phase(phase: f32) -> f32 {
    lerp(
        crate::core::NPC_PASS_HESITATION_MIN,
        crate::core::NPC_PASS_HESITATION_MAX,
        phase,
    )
}

fn lerp(min: f32, max: f32, phase: f32) -> f32 {
    let t = phase.fract().clamp(0.0, 1.0);
    min + (max - min) * t
}
