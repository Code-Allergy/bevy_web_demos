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
use web_demos::overball::components::*;
use web_demos::overball::constants::*;
use web_demos::overball::game_over::GameOverPlugin;
use web_demos::overball::game_ui::GameUIPlugin;
use web_demos::overball::main_menu::MainMenuPlugin;
use web_demos::overball::pause_menu::PauseMenuPlugin;
use web_demos::overball::resources::*;
use web_demos::overball::states::*;
use web_demos::overball::systems::*;
use web_demos::overball::victory::VictoryPlugin;
use web_demos::{overball::game_ui::PopupMessage, DefaultPluginsWithCustomWindow};

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
    "Game: Elements of a game from my childhood".to_string()
}
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

// Bevy code

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    let mut app = App::new();
    app
        // Plugins
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
        .add_systems(OnEnter(AppState::Loading), (load_audio_assets, load_ball_assets))
        .add_systems(
            Update,
            check_assets_loaded.run_if(in_state(AppState::Loading)),
        )
        // Game state
        .add_systems(
            OnEnter(AppState::Game),
            (
                reset_transition,
                setup_background_music.before(reset_transition),
            ),
        )
        .add_systems(
            OnEnter(InGameState::Reset),
            (setup_map, setup_player, clear_context).in_set(GameplaySet::Setup),
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
        .add_systems(OnEnter(InGameState::PlayerDied), handle_player_death);

    #[cfg(target_arch = "x86_64")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

fn configure_system_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (GameplaySet::Update
            .run_if(in_state(AppState::Game))
            .run_if(in_state(InGameState::Playing)),),
    );
    app.configure_sets(OnEnter(AppState::Game), GameplaySet::Setup);
}

// Transition system to start game when we enter AppState::Game
fn reset_transition(mut next_state: ResMut<NextState<InGameState>>) {
    next_state.set(InGameState::Reset);
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Flat map
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
    commands
        .spawn((
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

    // WinningTile
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
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 100_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),

            ..default()
        },
        GameMap,
    ));
}

fn setup_player(
    mut commands: Commands,
    ball_asset: Res<BallAsset>,
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
            scene_bundle: SceneBundle {
                scene: ball_asset.model.clone(),
                transform: Transform {
                    translation: ball_properties.position,
                    ..default()
                },
                ..default()
            },
            collider: Collider::ball(ball_properties.radius * 2.0),
            restitution: Restitution::coefficient(0.3),
            rigid_body: RigidBody::Dynamic,
        })
        .insert(ActiveEvents::COLLISION_EVENTS);

    // Player camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
            ..default()
        },
        PlayerCamera,
    ));

    game_state.set(AppState::Game);
    gameplay_state.set(InGameState::Playing);
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
    asset_server: Res<AssetServer>,
    audio_assets: Res<AudioAssets>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            if let Ok((mut ball, ball_transform)) = ball_query.get_mut(*entity2) {
                if let Ok((door_entity, door)) = door_query.get_mut(*entity1) {
                    ball.velocity = Vec3::ZERO;
                    debug!(
                        "Ball collided with door at position: {:?}",
                        ball_transform.translation
                    );
                    // Door thunk sound
                    commands.spawn(AudioBundle {
                        source: audio_assets.door_thunk_sound.clone(),
                        settings: PlaybackSettings {
                            volume: Volume::new(0.2),
                            ..default()
                        },
                    });

                    // Check if the player has the required score to open the door
                    if context.score >= door.required_score {
                        commands.spawn(AudioBundle {
                            source: audio_assets.door_opening_sound.clone(),
                            settings: PlaybackSettings {
                                volume: Volume::new(0.2),
                                ..default()
                            },
                        });

                        commands
                            .entity(door_entity)
                            .insert(DoorMovement { speed: 0.5 });
                    } else {
                        PopupMessage::spawn(
                            &mut commands,
                            &asset_server,
                            &format!(
                                "You need a score of {} to open this door",
                                door.required_score
                            ),
                            3.0,
                        );
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


