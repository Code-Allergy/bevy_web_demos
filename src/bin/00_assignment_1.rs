//! Illustrates the use of vertex colors.
//! The cube rotates around the y-axis and moves up and down.

// only used for exporting the demo name, and packaging the wasm
use wasm_bindgen::prelude::*;


use bevy::{prelude::*, render::mesh::VertexAttributeValues};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // primary_window: Some(get_window()),
        ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_cube)
        .add_systems(Update, (move_cube_up_and_down, update_colour))
        .run();
}

#[wasm_bindgen]
pub fn demo_name() -> String {
    "Assignment 1: Replication".to_string()
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // set cube mesh and colours
    let mut colorful_cube = Mesh::from(Cuboid::default());
    if let Some(VertexAttributeValues::Float32x3(positions)) = 
    colorful_cube.attribute(Mesh::ATTRIBUTE_POSITION) {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[r, g, b]| 
                [(1. - *r) / 2.,
                 (1. - *g) / 2., 
                 (1. - *b) / 2., 1.]
            )
            .collect();
        colorful_cube.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }

    // Cube
    commands.spawn((PbrBundle {
        mesh: meshes.add(colorful_cube),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        }),
        ..default()
    }, Cube));

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn move_cube_up_and_down(time: Res<Time>, mut query: Query<&mut Transform, With<Cube>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y = (time.elapsed_seconds().sin() / 2.0) as f32;
    }
}

fn update_colour(time: Res<Time>, 
    mut query: Query<(&Cube, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    for (_cube, material_handle) in query.iter_mut() {
        let brightness = (time.elapsed_seconds() as f32).sin() * 0.5 + 0.5; // Scale and shift to [0, 1]
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = Color::srgb(brightness, brightness, brightness); // Set the color with brightness
        }
    }
}


fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<Cube>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        for mut transform in query.iter_mut() {
            transform.rotate_y(time.delta_seconds() * 0.5);
            transform.rotate_x(time.delta_seconds() * 0.3);
        }
    }
}
