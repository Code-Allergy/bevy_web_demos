use bevy::app::{App, Startup, Update};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use wasm_bindgen::prelude::wasm_bindgen;
use web_demos::{GameControlPlugin, log};
use web_demos::player::PlayerPlugin;

#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String {
    "Loading small model".to_string()
}


#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    #[cfg(target_arch = "wasm32")]
    log("Starting game");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            #[cfg(target_arch = "wasm32")]
            primary_window: Some(Window {
                canvas: Some("#game-window".into()),
                ..default()
            }),

            ..default()
        }))
        .add_systems(Startup, setup)
        .add_plugins(GameControlPlugin)
        // .add_plugins(PlayerPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 300_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 4.0, 2.0),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("models/Avocado.glb")),
        transform: Transform {
            scale: Vec3::splat(16.0),
            ..default()
        },
        ..default()
    });
}

fn main() {
    #[cfg(target_arch = "x86_64")]
    start_game();
}