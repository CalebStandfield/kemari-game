use bevy::prelude::*;
use bevy::window::{PresentMode, Window};

use crate::core::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};

pub fn clear_color() -> Color {
    Color::srgb(0.08, 0.10, 0.11)
}

pub fn primary_window() -> Window {
    Window {
        title: WINDOW_TITLE.to_string(),
        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
        resizable: false,
        present_mode: PresentMode::Fifo,
        ..default()
    }
}
