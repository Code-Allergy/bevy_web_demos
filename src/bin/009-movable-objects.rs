use bevy::app::{App, Startup};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::DefaultPlugins;
use bevy::input::mouse::MouseButtonInput;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::render::mesh::{PlaneMeshBuilder, SphereMeshBuilder, VertexAttributeValues};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use wasm_bindgen::prelude::*;
use web_demos::{DefaultPluginsWithCustomWindow};
use web_demos::player::PlayerPlugin;

#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String { include_str!("009-movable-objects.rs").to_string() }
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String { "Pickup Objects".to_string() }
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

// Start of code

// Distance to allow pickup of objects
const PICKUP_DISTANCE: f32 = 10.0;
// Distance to hold object in front of camera
const HOLD_DISTANCE: f32 = 5.0;
// Movement speed of picked up object
const MOVEMENT_SPEED: f32 = 10.0;
// Size of the ground plane
const PLANE_SIZE: f32 = 200.0;
// Cone of pickup range
const MIN_ALIGNMENT: f32 = 0.85;
// Maximum speed of released object
const MAX_RELEASE_SPEED: f32 = 10.0;


// Apply a color to the object based on its state for the demo
const REGULAR_COLOR: Color = Color::WHITE;
const PICKABLE_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
const PICKED_UP_COLOR: Color = Color::srgb(0.2, 0.2, 0.8);

// World plane
#[derive(Component)]
struct Map;

// Mark entities that can be picked up
#[derive(Component)]
struct Pickable;

// Track the picked up state
#[derive(Component)]
struct PickedUp {
    previous_velocity: Vec3,
}

// Handle state transitions
#[derive(Component)]
struct PickupIntent;

#[derive(Component)]
struct ReleaseIntent;

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    App::new()
        .add_plugins(DefaultPluginsWithCustomWindow)
        .add_plugins(PlayerPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            pickup_detection,
            handle_pickup.after(pickup_detection),
            handle_release.after(pickup_detection),
            move_picked_object,
            update_object_colors
            )
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Ground plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(PlaneMeshBuilder::from_length(PLANE_SIZE)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(PLANE_SIZE/2.0, 0.0, PLANE_SIZE/2.0),
        Map,
    ));

    // Pickable ball
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere::new(0.5))),
            material: materials.add(Color::srgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(4.0, 0.5, 2.0),
            ..default()
        },
        Velocity::default(),
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Pickable,
    ));

    // Pickable cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5))),
            material: materials.add(Color::srgb(0.2, 0.2, 0.8)),
            transform: Transform::from_xyz(-4.0, 0.5, 2.0),
            ..default()
        },
        Velocity::default(),
        RigidBody::Dynamic,
        Collider::cuboid(0.25, 0.25, 0.25),
        Pickable,
    ));

    // Non-Pickable ball
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere::new(0.5))),
            material: materials.add(Color::srgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(4.0, 0.5, -2.0),
            ..default()
        },
        Velocity::default(),
        RigidBody::Dynamic,
        Collider::ball(0.5),
    ));

    // Non-Pickable cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5))),
            material: materials.add(Color::srgb(0.2, 0.2, 0.8)),
            transform: Transform::from_xyz(-4.0, 0.5, -2.0),
            ..default()
        },
        Velocity::default(),
        RigidBody::Dynamic,
        Collider::cuboid(0.25, 0.25, 0.25),
    ));
}

// Detect pickup and release intents
// System to detect pickup and release intentions
fn pickup_detection(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    pickable_q: Query<(Entity, &Transform), (With<Pickable>, Without<PickedUp>)>,
    picked_up_q: Query<Entity, With<PickedUp>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(cursor_position) = window.cursor_position() {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                for (entity, transform) in pickable_q.iter() {
                    // Calculate the vector from ray origin to the object
                    let to_object = transform.translation - ray.origin;
                    let distance = to_object.length();

                    // Skip if too far away
                    if distance > PICKUP_DISTANCE { continue; }

                    // Calculate how aligned the object is with our view direction
                    let direction = ray.direction.normalize();
                    let to_object_normalized = to_object.normalize();

                    let alignment = direction.dot(to_object_normalized);
                    if alignment > MIN_ALIGNMENT {
                        println!("Pickup: {:?}", entity);
                        commands.entity(entity).insert(PickupIntent);
                    }
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        for entity in picked_up_q.iter() {
            commands.entity(entity).insert(ReleaseIntent);
        }
    }
}

// Handle pickup intent and transition to picked up state
fn handle_pickup(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Velocity), With<PickupIntent>>,
) {
    for (entity, transform, velocity) in query.iter_mut() {
        commands.entity(entity)
            .remove::<PickupIntent>()
            .insert(PickedUp {
                previous_velocity: velocity.linvel,
            })
            .insert(RigidBody::KinematicPositionBased);
    }
}

// Handle release intent and transition to regular state
fn handle_release(
    mut commands: Commands,
    mut query: Query<(Entity, &PickedUp), With<ReleaseIntent>>,
) {
    for (entity, picked_up) in query.iter() {
        let release_velocity = picked_up.previous_velocity.clamp_length_max(MAX_RELEASE_SPEED);

        commands.entity(entity)
            .remove::<ReleaseIntent>()
            .remove::<PickedUp>()
            .insert(RigidBody::Dynamic)
            .insert(Velocity {
                linvel: release_velocity,
                angvel: Vec3::ZERO,
            });
    }
}

// Move picked up objects to hover infront of the camera at the crosshair
fn move_picked_object(
    camera_q: Query<&GlobalTransform, With<Camera>>,
    mut picked_up_q: Query<(&mut Transform, &mut Velocity), With<PickedUp>>,
    time: Res<Time>,
) {
    let camera_transform = camera_q.single();
    let target_position = camera_transform.translation() + camera_transform.forward() * HOLD_DISTANCE;

    for (mut transform, mut velocity) in picked_up_q.iter_mut() {
        // Calculate movement needed to reach the cursor ray position
        let movement = target_position - transform.translation;

        // move the object smoothly to the cursor position
        transform.translation += movement * MOVEMENT_SPEED * time.delta_seconds();
        velocity.linvel = movement * MOVEMENT_SPEED;
    }
}

// System to update colors based on components
fn update_object_colors(
    mut query: Query<(
        &mut Handle<StandardMaterial>,
        Option<&Pickable>,
        Option<&PickedUp>,
        Option<&Map>
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut material_handle, pickable, picked_up, map) in query.iter_mut() {
        // Skip recolouring map, only colour pickable and picked up objects
        if map.is_some() {
            continue;
        }

        let new_color = if pickable.is_some() && picked_up.is_some() {
            PICKED_UP_COLOR
        } else if pickable.is_some() {
            PICKABLE_COLOR
        } else {
            REGULAR_COLOR
        };

        if let Some(material) = materials.get_mut(&*material_handle) {
            material.base_color = new_color;
        }
    }
}