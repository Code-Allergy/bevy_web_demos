use bevy::app::{App, Startup};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use wasm_bindgen::prelude::wasm_bindgen;
use web_demos::{DefaultPluginsWithCustomWindow};
use web_demos::player::PlayerPlugin;

#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String { include_str!("002-basic-models.rs").to_string() }
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String { "A small glFW model".to_string() }
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

// BEVY CODE
#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    App::new()
        .add_plugins(DefaultPluginsWithCustomWindow)
        .add_systems(Startup, setup)
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

    // Lights
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 300_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 4.0, 2.0),
        ..default()
    });

    // Model
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