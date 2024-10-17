use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Loading,
    Title,
    Game,
}

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum InGameState {
    #[default]
    NotInGame,
    Reset,
    Playing,
    Paused,
    PlayerDied,
    GameOver,
    Victory,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MainMenuSet {
    Setup,
    Update,
    Cleanup,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameplaySet {
    Setup,
    Update,
}
