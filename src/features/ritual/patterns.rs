use bevy::prelude::*;

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct TouchPatternState {
    pub last_touch: Option<crate::core::TouchKind>,
    pub repeat_count: u32,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PossessionJuggleRhythmState {
    pub holder: Option<Entity>,
    pub juggle_count: u8,
}
