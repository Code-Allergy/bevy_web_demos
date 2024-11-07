use bevy::prelude::*;
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

const PLANE_SIZE: f32 = 20.0;
const BALL_SIZE: f32 = 1.0;
const BALL_HEIGHT: f32 = 10.0;


fn setup(mut commands: Commands,
                 mut meshes: ResMut<Assets<Mesh>>,
                 mut materials: ResMut<Assets<StandardMaterial>>) {
    // Plane Mesh
    commands.spawn(
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid {
                half_size: Vec3::new(PLANE_SIZE/2.0, 0.1, PLANE_SIZE/2.0),
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),  // Ground color
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.05, 0.0),
            ..default()
        })
        .insert((Collider::cuboid(PLANE_SIZE/2.0, 0.1, PLANE_SIZE/2.0),
                 Restitution::coefficient(0.9)));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 300_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 0.0),
        ..default()
    });

    // Add colliders around the edges of the ground plane
    let wall_thickness = 0.1;
    let wall_height = 10000.0;
    let half_plane_size = PLANE_SIZE / 2.0;

    // Left wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(wall_thickness, wall_height, half_plane_size),
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        }),
        transform: Transform::from_xyz(-half_plane_size - wall_thickness, wall_height, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(wall_thickness, wall_height, half_plane_size));

    // Right wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(wall_thickness, wall_height, half_plane_size),
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        }),
        transform: Transform::from_xyz(half_plane_size + wall_thickness, wall_height, 0.0),
        ..default()
    })
        .insert(Collider::cuboid(wall_thickness, wall_height, half_plane_size));

    // Front wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(half_plane_size, wall_height, wall_thickness),
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, wall_height, half_plane_size + wall_thickness),
        ..default()
    })
        .insert(Collider::cuboid(half_plane_size, wall_height, wall_thickness));

    // Back wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(half_plane_size, wall_height, wall_thickness),
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, wall_height, -half_plane_size - wall_thickness),
        ..default()
    })
        .insert(Collider::cuboid(half_plane_size, wall_height, wall_thickness));

    spawn_balls(&mut commands, &mut meshes, &mut materials, 1000);
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

        spawn_balls(&mut commands, &mut meshes, &mut materials, 1000);
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