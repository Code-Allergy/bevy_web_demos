use bevy::prelude::*;

use super::constants::*;

#[derive(Resource, Debug)]
pub struct GameContext {
    pub lives: u32,
    pub score: u32,
    pub level: u32,
}

impl Default for GameContext {
    fn default() -> Self {
        GameContext {
            lives: PLAYER_LIVES,
            score: 0,
            level: 1,
        }
    }
}

impl GameContext {
    pub fn reset(&mut self) {
        self.lives = PLAYER_LIVES;
        self.score = 0;
        self.level = 1;
    }
}

#[derive(Resource)]
pub struct AudioAssets {
    pub bg_music: Handle<AudioSource>,
    pub game_over_sound: Handle<AudioSource>,
    pub door_thunk_sound: Handle<AudioSource>,
    pub door_opening_sound: Handle<AudioSource>,
    pub victory_sound: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct BallAsset {
    pub model: Handle<Scene>,
}
