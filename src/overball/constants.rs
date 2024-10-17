use bevy::prelude::*;
use bevy::color::Color;

pub const MIN_X: f32 = -20.0;
pub const MAX_X: f32 = 20.0;
pub const MIN_Y: f32 = -10.0; // Keep Y > 0 to prevent falling out of the world
pub const MAX_Y: f32 = 10.0;
pub const MIN_Z: f32 = -20.0;
pub const MAX_Z: f32 = 20.0;

pub const PLAYER_LIVES: u32 = 0;

pub const MOVEMENT_SPEED: f32 = 2.0;
pub const DAMPING_FACTOR: f32 = 0.7;

pub const DOOR_POSITION: Vec3 = Vec3::new(10.5, 0.5, 0.0);
pub const DOOR_REQUIRED_SCORE: u32 = 10;

// UI Style
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
