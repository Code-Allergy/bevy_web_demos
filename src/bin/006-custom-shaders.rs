//! A shader that uses the GLSL shading language.

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use wasm_bindgen::prelude::wasm_bindgen;
use web_demos::DefaultPluginsWithCustomWindow;

#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String {
    include_str!("006-custom-shaders.rs").to_string()
}
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String {
    "Shaders: Custom shaders (Still Broken!)".to_string()
}
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    App::new()
        .add_plugins((
            DefaultPluginsWithCustomWindow,
            MaterialPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_and_orbit_model, update_mode))
        .run();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
enum ShaderStyle {
    None,
    Phong,
    Cartoon,
    Gooch,
}

impl ShaderStyle {
    fn as_str(&self) -> &'static str {
        match self {
            ShaderStyle::None => "None",
            ShaderStyle::Phong => "Phong",
            ShaderStyle::Cartoon => "Cartoon",
            ShaderStyle::Gooch => "Gooch",
        }
    }
}

const ORBIT_RADIUS: f32 = 2.0;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
        ..default()
    };
    // cube
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Cuboid::default()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            material: materials.add(CustomMaterial {
                ambient_color: Vec4::new(0.1, 0.1, 0.1, 1.0),
                diffuse_color: Vec4::new(0.4, 0.0, 0.0, 1.0),
                specular_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                shininess: 10.0,
                ..default()
            }),
            ..default()
        },
        Model,
    ));

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(4.0, 2.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section("Shading: None", text_style.clone()),
                ShadingText,
            ));
        });
}

#[derive(Component)]
struct Model;
#[derive(Component)]
struct ShadingText;

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Default, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    pub ambient_color: Vec4,
    #[uniform(1)]
    pub diffuse_color: Vec4,
    #[uniform(2)]
    pub specular_color: Vec4,
    #[uniform(3)]
    pub shininess: f32,
    #[uniform(3)]
    pub _wasm_padding1: Vec3,
    #[uniform(4)]
    pub mode: i32,
    #[uniform(4)]
    pub _wasm_padding2: Vec3,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overridden.
impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/vertex.vert".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/fragment.frag".into()
    }

    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}

// SYSTEMS

fn rotate_and_orbit_model(time: Res<Time>, mut query: Query<(&Model, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        // Rotate the model around its own axis
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));

        // Calculate orbit position
        let angle = time.elapsed_seconds() * 0.5; // Adjust speed as needed
        let x = ORBIT_RADIUS * angle.cos();
        let z = ORBIT_RADIUS * angle.sin();

        // Update the translation for orbiting
        transform.translation = Vec3::new(x, transform.translation.y, z);
    }
}

fn update_mode(
    mut materials: ResMut<Assets<CustomMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<&Handle<CustomMaterial>>,
    mut text_query: Query<&mut Text, With<ShadingText>>,
) {
    for material_handle in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            let new_mode = if keyboard_input.just_pressed(KeyCode::Digit1) {
                Some(ShaderStyle::Phong)
            } else if keyboard_input.just_pressed(KeyCode::Digit2) {
                Some(ShaderStyle::Cartoon)
            } else if keyboard_input.just_pressed(KeyCode::Digit3) {
                Some(ShaderStyle::Gooch)
            } else if keyboard_input.just_pressed(KeyCode::Digit0) {
                Some(ShaderStyle::None)
            } else {
                None
            };

            if let Some(mode) = new_mode {
                material.mode = mode as i32;
                info!("Switched to {} shading mode", mode.as_str());

                // Update the UI text
                if let Ok(mut text) = text_query.get_single_mut() {
                    text.sections[0].value = format!("Shading: {}", mode.as_str());
                }
            }
        }
    }
}
