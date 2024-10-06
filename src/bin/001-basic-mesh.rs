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
pub fn source_file() -> String { include_str!("001-basic-mesh.rs").to_string() }
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String { "A basic cube using bevy primitives".to_string() }
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

// BEVY CODE

/* DefaultPluginWithCustomWindow:
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        #[cfg(target_arch = "wasm32")]
        primary_window: Some(Window {
            canvas: Some("#game-window".into()),
            ..default()
        }),
        ..default()
    }));
 */

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    App::new()
        .add_plugins(DefaultPluginsWithCustomWindow)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(4.0, 2.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Basic cube mesh
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::default())),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),
            ..default()
        }),
        ..default()
    });
}