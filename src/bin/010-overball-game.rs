use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use wasm_bindgen::prelude::*;

use bevy::{
    app::App,
    asset::Assets,
    audio::Volume,
    color::Color,
    math::Vec3,
    pbr::{PbrBundle, StandardMaterial},
    render::mesh::PlaneMeshBuilder,
};
use web_demos::DefaultPluginsWithCustomWindow;
use web_demos::overball::components::*;
use web_demos::overball::systems::*;
use web_demos::overball::constants::*;
use web_demos::overball::states::*;
use web_demos::overball::resources::*;
use web_demos::overball::main_menu::MainMenuPlugin;
use web_demos::overball::pause_menu::PauseMenuPlugin;
use web_demos::overball::game_ui::GameUIPlugin;
use web_demos::overball::game_over::GameOverPlugin;
use web_demos::overball::victory::VictoryPlugin;

// DEBUG
// use web_demos::overball::util::*;

// Debug only on x86_64
#[cfg(target_arch = "x86_64")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String {
    include_str!("010-overball-game.rs").to_string()
}
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String {
    "A basic game from my childhood".to_string()
}
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    let mut app = App::new();
    app
        // Plugins
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(DefaultPluginsWithCustomWindow)

        // My plugins
        .add_plugins(MainMenuPlugin)
        .add_plugins(PauseMenuPlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(VictoryPlugin)
        .add_plugins(GameOverPlugin);

    // States
    app.insert_state(AppState::Loading)
        .init_state::<InGameState>()
        .init_resource::<GameContext>();

    configure_system_sets(&mut app);

    // Add systems to sets
    app
        // Loading state
        .add_systems(OnEnter(AppState::Loading), load_audio_assets)
        .add_systems(
            Update,
            check_audio_loaded.run_if(in_state(AppState::Loading)),
        )

        // Game state
        .add_systems(
            OnEnter(AppState::Game),
            (
                reset_transition,
                setup_background_music.before(reset_transition), // TODO we should have a method to disable audio
            ),
        )
        .add_systems(
            OnEnter(InGameState::Reset),
            (
                setup_map,
                setup_player,
                clear_context,
            )
                .in_set(GameplaySet::Setup),
        )
        .add_systems(
            Update,
            (
                // Player
                move_player_when_pressing_keys,
                check_player_out_of_bounds,

                // Door
                handle_door_collisions,
                update_door_movement,

                // Tile
                detect_ball_on_tile,
                check_winning_tile,
            )
                .in_set(GameplaySet::Update)
                .run_if(in_state(InGameState::Playing)),
        )
        // Player Died
        .add_systems(OnEnter(InGameState::PlayerDied), handle_player_death)
    ;

    #[cfg(target_arch = "x86_64")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

fn configure_system_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            GameplaySet::Update
                .run_if(in_state(AppState::Game))
                .run_if(in_state(InGameState::Playing)),
        ),
    );
    app.configure_sets(OnEnter(AppState::Game), GameplaySet::Setup);
}

// Transition system to start game when we enter AppState::Game
fn reset_transition(
    mut next_state: ResMut<NextState<InGameState>>,
) {
    next_state.set(InGameState::Reset);
}

// Loading
fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bg_music = asset_server.load("sounds/bg.mp3");
    let game_over_sound = asset_server.load("sounds/game_over.wav");

    commands.insert_resource(AudioAssets {
        bg_music,
        game_over_sound,
    });
}

// TODO change this
fn check_audio_loaded(
    asset_server: Res<AssetServer>,
    audio_assets: Res<AudioAssets>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    if asset_server.get_load_state(&audio_assets.bg_music) == Some(bevy::asset::LoadState::Loaded)
        && asset_server.get_load_state(&audio_assets.game_over_sound)
            == Some(bevy::asset::LoadState::Loaded)
    {
        game_state.set(AppState::Title);
    }
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // flat plane for testing
    commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Collider::cuboid(10.0, 0.1, 10.0),
            Restitution::coefficient(0.9),
            InheritedVisibility::default(),
            GameMap,
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(PlaneMeshBuilder::from_length(20.0)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 1.0),
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..default()
            });
        });

    // WinningTile Platform
    commands.spawn((
        Transform::from_xyz(15.0, 0.0, 0.0),
        GlobalTransform::default(),
        Collider::cuboid(5.0, 0.1, 2.5),
        Restitution::coefficient(0.9),
        InheritedVisibility::default(),
        GameMap,
    ))
    .with_children(|parent| {
        parent.spawn(PbrBundle {
            mesh: meshes.add(PlaneMeshBuilder::from_size(Vec2::new(10.0, 5.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.0),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.1, 0.0),
            ..default()
        });
    });

    // Winning Tile
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(PlaneMeshBuilder::from_length(4.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 1.0, 0.0),
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(15.0, 0.1, 0.0)),
            ..default()
        },
        WinningTile,
        GameMap,
    ));


    // Tiles
    for x in -5..=5 {
        for z in -5..=5 {
            let position = Vec3::new(x as f32, 0.1, z as f32);
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(PlaneMeshBuilder::from_length(0.5)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(0.5, 0.5, 0.5),
                        ..default()
                    }),
                    transform: Transform::from_translation(position),
                    ..default()
                },
                Tile {
                    position,
                    activated: false,
                },
                GameMap,
            ));
        }
    }

    // Door
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 5.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 1.0),
                ..default()
            }),
            transform: Transform::from_translation(DOOR_POSITION),
            ..default()
        },
        Collider::cuboid(0.5, 0.5, 2.5),
        Door {
            required_score: DOOR_REQUIRED_SCORE,
            is_open: false,
        },
        GameMap,
    ));

    // light
    commands.spawn((PointLightBundle {
        point_light: PointLight {
            intensity: 100_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),

        ..default()
    }, GameMap));
}


fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state: ResMut<NextState<AppState>>,
    mut gameplay_state: ResMut<NextState<InGameState>>,
) {
    let ball_properties = BallProperties::default();
    // Player ball
    commands
        .spawn(PlayerBundle {
            player: Player,
            ball: Ball {
                position: ball_properties.position,
                velocity: ball_properties.velocity,
                radius: ball_properties.radius,
            },
            pbr_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(Sphere::new(ball_properties.radius * 2.0))),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.0, 0.0),
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, ball_properties.radius * 2.0, 0.0),
                ..default()
            },
            collider: Collider::ball(ball_properties.radius * 2.0),
            restitution: Restitution::coefficient(0.3),
            rigid_body: RigidBody::Dynamic,
        })
        .insert(ActiveEvents::COLLISION_EVENTS);

    // Player camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ..default()
    }, PlayerCamera));

    game_state.set(AppState::Game);
    gameplay_state.set(InGameState::Playing);
}

fn setup_background_music(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn(AudioBundle {
        source: audio_assets.bg_music.clone(),
        settings: PlaybackSettings {
            volume: Volume::new(0.2),
            ..default()
        },
    });
}

fn clear_context(
    mut context: ResMut<GameContext>,
) {
    context.reset();
}

// BEVY CODE

#[derive(Debug)]
struct BallProperties {
    radius: f32,
    position: Vec3,
    velocity: Vec3,
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
struct PlayerBundle {
    player: Player,
    ball: Ball,
    pbr_bundle: PbrBundle,
    collider: Collider,
    restitution: Restitution,
    rigid_body: RigidBody,
}

#[derive(Component)]
struct DoorMovement {
    speed: f32,
}

fn handle_door_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    context: Res<GameContext>,
    mut ball_query: Query<(&mut Ball, &Transform), With<Player>>,
    mut door_query: Query<(Entity, &Door)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            if let Ok((mut ball, ball_transform)) = ball_query.get_mut(*entity2) {
                if let Ok((door_entity, door)) = door_query.get_mut(*entity1) {
                    ball.velocity = Vec3::ZERO;
                    info!("Ball collided with door at position: {:?}", ball_transform.translation);
                    if context.score > door.required_score {
                        commands.entity(door_entity).insert(DoorMovement { speed: 1.0 });
                    } else {
                        // alert the player that they need more points
                    }
                }
            }
        }
    }
}

fn update_door_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &DoorMovement)>,
) {
    for (entity, mut transform, door_movement) in query.iter_mut() {
        transform.translation.y -= door_movement.speed * time.delta_seconds();
        if transform.translation.y < 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}



fn check_winning_tile(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    winning_tile_query: Query<&Transform, With<WinningTile>>,
    mut timer_query: Query<(Entity, &mut WinningTileTimer)>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;

        for winning_tile_transform in winning_tile_query.iter() {
            let tile_position = winning_tile_transform.translation;

            // Check if the player is on the winning tile
            if (player_position.x - tile_position.x).abs() < 2.5 &&
               (player_position.z - tile_position.z).abs() < 5.0 {
                // If the timer already exists, update it
                if let Ok((entity, mut timer)) = timer_query.get_single_mut() {
                    timer.0.tick(time.delta());
                    if timer.0.finished() {
                        next_state.set(InGameState::Victory);
                        commands.entity(entity).despawn(); // Remove the timer entity
                    }
                } else {
                    // If the timer doesn't exist, create it
                    commands.spawn(WinningTileTimer(Timer::from_seconds(2.0, TimerMode::Once)));
                }
                return;
            }
        }
    }

    // If the player is not on the winning tile, reset the timer
    if let Ok((entity, _)) = timer_query.get_single() {
        commands.entity(entity).despawn();
    }
}
