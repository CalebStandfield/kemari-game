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
    RecoveringBall,
    ChoosingPass,
    ExecutingPass,
    Passing,
    RecoveringToHome,
}

#[derive(Component, Debug, Clone)]
pub struct NpcBehavior {
    pub state: NpcBehaviorState,
    pub drift_target: Vec2,
    pub drift_timer: Timer,
    pub call_decision_timer: Timer,
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
            last_pass_target: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ControllerActionState {
    Idle,
    PreparingToReceive,
    ControllingBall,
    RecoveringBall,
    ChoosingPass,
    ExecutingPass,
}

#[derive(Component, Debug, Clone)]
pub struct NpcControllerPlan {
    pub action: ControllerActionState,
    pub settle_timer: Timer,
    pub decision_timer: Timer,
    pub execution_timer: Timer,
    pub pending_target: Option<Entity>,
    pub pending_touch_kind: Option<crate::core::TouchKind>,
}

impl NpcControllerPlan {
    pub fn new(phase: f32) -> Self {
        Self {
            action: ControllerActionState::Idle,
            settle_timer: timer_once(controller_settle_delay_from_phase(phase)),
            decision_timer: timer_once(pass_decision_delay_from_phase(phase)),
            execution_timer: timer_once(crate::core::NPC_PASS_EXECUTION_DELAY),
            pending_target: None,
            pending_touch_kind: None,
        }
    }

    pub fn begin_receive(&mut self, phase: f32) {
        self.action = ControllerActionState::PreparingToReceive;
        self.pending_target = None;
        self.pending_touch_kind = None;
        self.settle_timer = timer_once(controller_settle_delay_from_phase(phase));
    }

    pub fn reset_idle(&mut self, phase: f32) {
        self.action = ControllerActionState::Idle;
        self.pending_target = None;
        self.pending_touch_kind = None;
        self.settle_timer = timer_once(controller_settle_delay_from_phase(phase));
        self.decision_timer = timer_once(pass_decision_delay_from_phase(phase));
        self.execution_timer = timer_once(crate::core::NPC_PASS_EXECUTION_DELAY);
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

pub fn controller_settle_delay_from_phase(phase: f32) -> f32 {
    lerp(
        crate::core::NPC_CONTROLLER_SETTLE_DELAY_MIN,
        crate::core::NPC_CONTROLLER_SETTLE_DELAY_MAX,
        phase,
    )
}

pub fn pass_decision_delay_from_phase(phase: f32) -> f32 {
    lerp(
        crate::core::NPC_PASS_DECISION_DELAY_MIN,
        crate::core::NPC_PASS_DECISION_DELAY_MAX,
        phase,
    )
}

fn timer_once(seconds: f32) -> Timer {
    Timer::from_seconds(seconds.max(0.001), TimerMode::Once)
}

fn lerp(min: f32, max: f32, phase: f32) -> f32 {
    let t = phase.fract().clamp(0.0, 1.0);
    min + (max - min) * t
}
