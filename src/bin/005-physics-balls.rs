use bevy::prelude::*;
use bevy::render::mesh::PlaneMeshBuilder;
use bevy_rapier3d::prelude::*;
use wasm_bindgen::prelude::*;
use web_demos::{player::PlayerPlugin, DefaultPluginsWithCustomWindow};
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String {
    "Physics Demo - Ball Pit".to_string()
}
#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String { include_str!("005-physics-balls.rs").to_string() }
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

// BEVY CODE

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    App::new()
        .add_plugins(DefaultPluginsWithCustomWindow)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, respawn_balls)

        .run();
}

const PLANE_SIZE: f32 = 25.0;
const BALL_SIZE: f32 = 1.0;
const BALL_HEIGHT: f32 = 10.0;
const TOTAL_BALLS: u32 = 2500;


fn setup(mut commands: Commands,
                 mut meshes: ResMut<Assets<Mesh>>,
                 mut materials: ResMut<Assets<StandardMaterial>>) {
    // Plane Mesh
    commands.spawn(
        PbrBundle {
            mesh: meshes.add(PlaneMeshBuilder::from_length(PLANE_SIZE)),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),  // Ground color
            transform: Transform::from_xyz(0.0, 0.05, 0.0),
            ..default()
        })
        .insert((Collider::cuboid(PLANE_SIZE/2.0, 0.1, PLANE_SIZE/2.0),
                 Restitution::coefficient(0.9)));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            shadow_depth_bias: 0.1,
            shadow_normal_bias: 0.1,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 50.0, 0.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    spawn_walls(&mut commands, &mut meshes, &mut materials);
    spawn_balls(&mut commands, &mut meshes, &mut materials, TOTAL_BALLS);
}

#[derive(Component, Default)]
struct Ball;

#[derive(Bundle, Default)]
struct BallBundle {
    pbr_bundle: PbrBundle,
    collider: Collider,
    restitution: Restitution,
    rigid_body: RigidBody,
    velocity: Velocity,
    damping: Damping,
    friction: Friction,
    ball: Ball,
}

fn respawn_balls(mut commands: Commands, query: Query<Entity, With<Ball>>, keyboard: Res<ButtonInput<KeyCode>>,
                 mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        spawn_balls(&mut commands, &mut meshes, &mut materials, TOTAL_BALLS);
    }
}

fn spawn_balls(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, count: u32) {
    let mut balls_vec: Vec<BallBundle> = Vec::with_capacity(count as usize);
    for i in 0..count {
        let x = rand::random();
        let z = rand::random();
        let height = BALL_HEIGHT + (i as f32 * 0.01) * 4.0;
        let ball = BallBundle {
            pbr_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(Sphere { radius: BALL_SIZE / 2.0 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.2, 0.3),
                    ..default()
                }),
                transform: Transform::from_xyz(x, height, z),
                ..default()
            },
            collider: Collider::ball(BALL_SIZE / 2.0),
            restitution: Restitution::coefficient(0.3),
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::default(),
            damping: Damping::default(),
            friction: Friction::default(),
            ball: Ball,
        };

        balls_vec.push(ball);
    }

    commands.spawn_batch(balls_vec);
}

#[derive(Debug)]
struct WallConfig {
    position: Vec3,
    half_size: Vec3,
}

fn spawn_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    const WALL_THICKNESS: f32 = 0.1;
    const WALL_HEIGHT: f32 = 10000.0;
    let half_plane_size = PLANE_SIZE / 2.0;

    let wall_configs = [
        // Left wall
        WallConfig {
            position: Vec3::new(-half_plane_size - WALL_THICKNESS, WALL_HEIGHT, 0.0),
            half_size: Vec3::new(WALL_THICKNESS, WALL_HEIGHT, half_plane_size),
        },
        // Right wall
        WallConfig {
            position: Vec3::new(half_plane_size + WALL_THICKNESS, WALL_HEIGHT, 0.0),
            half_size: Vec3::new(WALL_THICKNESS, WALL_HEIGHT, half_plane_size),
        },
        // Front wall
        WallConfig {
            position: Vec3::new(0.0, WALL_HEIGHT, half_plane_size + WALL_THICKNESS),
            half_size: Vec3::new(half_plane_size, WALL_HEIGHT, WALL_THICKNESS),
        },
        // Back wall
        WallConfig {
            position: Vec3::new(0.0, WALL_HEIGHT, -half_plane_size - WALL_THICKNESS),
            half_size: Vec3::new(half_plane_size, WALL_HEIGHT, WALL_THICKNESS),
        },
    ];

    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        ..default()
    });

    for config in wall_configs {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid {
                    half_size: config.half_size,
                })),
                material: wall_material.clone(),
                transform: Transform::from_translation(config.position),
                ..default()
            })
            .insert(Collider::cuboid(
                config.half_size.x,
                config.half_size.y,
                config.half_size.z,
            ));
    }
}
