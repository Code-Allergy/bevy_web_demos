use bevy::prelude::*;

pub mod player;

pub fn get_window() -> Window {
    Window {
        canvas: Some("#demo_canvas".into()),
        ..default()
    }
}