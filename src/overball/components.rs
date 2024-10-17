use bevy::prelude::*;

#[derive(Component)]
pub struct GameMap;

#[derive(Component)]
pub struct Tile {
    pub position: Vec3,
    pub activated: bool,
}

#[derive(Component)]
pub struct PlayerCamera;

// Win condition
#[derive(Component)]
pub struct WinningTile;

#[derive(Component)]
pub struct WinningTileTimer(pub Timer);

#[derive(Component)]
pub struct Door {
    pub required_score: u32,
    pub is_open: bool,
}

#[derive(Component)]
pub struct Ball {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
}

#[derive(Component)]
pub struct Player;
