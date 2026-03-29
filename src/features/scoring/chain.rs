use bevy::prelude::*;

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct ChainCounter {
    pub value: u32,
}
