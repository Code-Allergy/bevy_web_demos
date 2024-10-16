use bevy::app::{App, Startup};
use bevy::math::Vec3;
use bevy::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;
use web_demos::player::PlayerPlugin;
use web_demos::DefaultPluginsWithCustomWindow;

#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String {
    "Loading complex scene".to_string()
}
#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String {
    include_str!("004-complex-scene.rs").to_string()
}
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
        .add_plugins(PlayerPlugin)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            .load(GltfAssetLabel::Scene(0).from_asset("models/ABeautifulGame/ABeautifulGame.gltf")),
        transform: Transform {
            scale: Vec3::splat(16.0),
            ..default()
        },
        ..default()
    });
}
