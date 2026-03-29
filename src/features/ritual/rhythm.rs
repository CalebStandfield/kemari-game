use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct EleganceMeter {
    pub value: f32,
}

impl Default for EleganceMeter {
    fn default() -> Self {
        Self { value: 50.0 }
    }
}
