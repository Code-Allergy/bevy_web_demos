use bevy::prelude::*;
use bevy_rapier3d::dynamics::RigidBody;
use bevy_rapier3d::geometry::{Collider, Restitution};

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

#[derive(Debug)]
pub struct BallProperties {
    pub radius: f32,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Default for BallProperties {
    fn default() -> Self {
        BallProperties {
            radius: 0.2,
            position: Vec3::new(0.0, 1.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub ball: Ball,
    pub scene_bundle: SceneBundle,
    pub collider: Collider,
    pub restitution: Restitution,
    pub rigid_body: RigidBody,
}
