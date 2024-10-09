use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use wasm_bindgen::prelude::*;

use web_demos::DefaultPluginsWithCustomWindow;
use bevy::{
    app::App,
    asset::Assets,
    audio::Volume,
    color::{
        Color,
        palettes::basic::RED,
    },
    math::Vec3,
    pbr::{
        PbrBundle, 
        StandardMaterial
    },
    render::mesh::PlaneMeshBuilder,
};



// Debug only on x86_64
#[cfg(target_arch = "x86_64")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;


// Define the play area bounds
// Let the player clip off the playable area, but not fall off the world
const MIN_X: f32 = -20.0;
const MAX_X: f32 = 20.0;
const MIN_Y: f32 = -10.0;   // Keep Y > 0 to prevent falling out of the world
const MAX_Y: f32 = 10.0;
const MIN_Z: f32 = -20.0;
const MAX_Z: f32 = 20.0;

const PLAYER_LIVES: u32 = 0;


#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String { include_str!("010-overball-game.rs").to_string() }
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String { "A basic game from my childhood".to_string() }
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Loading,
    Title,
    Game,
}

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
enum InGameState {
    #[default]
    NotInGame,
    Playing,
    Paused,
    PlayerDied,
    GameOver,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MainMenuSet {
    Setup,
    Update,
    Cleanup,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameplaySet {
    Setup,
    Update,
}

// Preload audio assets
#[derive(Resource)]
struct AudioAssets {
    bg_music: Handle<AudioSource>,
    game_over_sound: Handle<AudioSource>,
}

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    let mut app = App::new();

    
    app
    
    // Plugins
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(DefaultPluginsWithCustomWindow);

    app.add_plugins( // TODO should pause when paused
        RapierPhysicsPlugin::<NoUserData>::default()); //.run_if(not(in_state(InGameState::Paused))));

    // app.add_systems(Update, display_events); // debug events
    // States
        
        app
        .insert_state(AppState::Loading)
        .init_state::<InGameState>();

    // Configure System Sets
    app.configure_sets(Update, (
        MainMenuSet::Update
            .run_if(in_state(AppState::Title)),
        GameplaySet::Update
            .run_if(in_state(AppState::Game))
            .run_if(in_state(InGameState::Playing)),
    ));

    // Main Menu
    app .configure_sets(OnEnter(AppState::Title), MainMenuSet::Setup)
        .configure_sets(OnExit(AppState::Title), MainMenuSet::Cleanup)

    // Game
        .configure_sets(OnEnter(AppState::Game), GameplaySet::Setup);

    // Add systems to sets
    app
        // Loading
        .add_systems(OnEnter(AppState::Loading), load_audio_assets)
        .add_systems(Update, check_audio_loaded.run_if(in_state(AppState::Loading)))

        // Main Menu
        .add_systems(OnEnter(AppState::Title), (setup_main_menu_ui,).in_set(MainMenuSet::Setup))
        .add_systems(Update, (start_button_system,).in_set(MainMenuSet::Update))
        .add_systems(OnExit(AppState::Title), (despawn_main_menu,).in_set(MainMenuSet::Cleanup))

        // Game
        .add_systems(OnEnter(AppState::Game), start_setup)


        .add_systems(OnEnter(InGameState::Playing), (
            setup_game_ui,
            setup_game_camera,
            setup_map,
            setup_player,
            setup_background_music // TODO we should have a method to disable audio
        ).in_set(GameplaySet::Setup))

        .add_systems(Update, (
            move_player_when_pressing_keys,
            check_player_out_of_bounds,
        ).in_set(GameplaySet::Update)
        .run_if(in_state(InGameState::Playing)))

        // Paused State
        .add_systems(OnEnter(InGameState::Paused), setup_pause_menu)
        .add_systems(OnExit(InGameState::Paused), despawn_pause_menu)
        .add_systems(Update, pause_game_input)

        // Player Died
        .add_systems(OnEnter(InGameState::PlayerDied), handle_player_death)
    

        // Game Over
        .add_systems(OnEnter(InGameState::GameOver), (
            setup_game_over_ui, play_gameover_sound 
        ))
        .add_systems(Update, handle_game_over_input.run_if(in_state(InGameState::GameOver)))
        .add_systems(OnExit(InGameState::GameOver), despawn_world);
        // .add_systems(Update, check_death_timer.run_if(in_state(InGameState::PlayerDied)));

        

        // Debug
    app

        .add_systems(Update, debug_game_state)
        .add_systems(Update, debug_in_game_state);

    #[cfg(target_arch = "x86_64")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

// Transition system to start game when we enter AppState::Game
fn start_setup(
    mut game_state: ResMut<NextState<InGameState>>,
) {
    game_state.set(InGameState::Playing);
}

// Loading
fn load_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
    if asset_server.get_load_state(&audio_assets.bg_music) == Some(bevy::asset::LoadState::Loaded) &&
        asset_server.get_load_state(&audio_assets.game_over_sound) == Some(bevy::asset::LoadState::Loaded)
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
    commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Collider::cuboid(10.0, 0.1, 10.0),
        Restitution::coefficient(0.9),
        InheritedVisibility::default()
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

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),

        ..default()
    });
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
    commands.spawn(PlayerBundle {
        lives: Lives { lives: PLAYER_LIVES },
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

    game_state.set(AppState::Game);
    gameplay_state.set(InGameState::Playing);
}

fn setup_background_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
) {
    commands.spawn(AudioBundle {
        source: audio_assets.bg_music.clone(),
        settings: PlaybackSettings {
            volume: Volume::new(0.2),
            ..default()
        },
    });
}

// this will later be spawned with the player, as it will track the player from behind
fn setup_game_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ..default()
    });
}

fn setup_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 28.0,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((TextBundle::from_section(
                "Lives: 3", text_style.clone(),
            ), LivesText));
        });
}

// BEVY CODE

#[derive(Debug)]
struct BallProperties {
    radius: f32,
    position: Vec3,
    velocity: Vec3,
}

// UI components
#[derive(Component)]
struct LivesText;

impl Default for BallProperties {
    fn default() -> Self {
        BallProperties {
            radius: 0.2,
            position: Vec3::new(0.0, 1.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Component)]
struct Ball {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Lives {
    pub lives: u32,
}

#[derive(Bundle)]
struct PlayerBundle {
    lives: Lives,
    player: Player,
    ball: Ball,
    pbr_bundle: PbrBundle,
    collider: Collider,
    restitution: Restitution,
    rigid_body: RigidBody,
}

const MOVEMENT_SPEED: f32 = 0.001;
const DAMPING_FACTOR: f32 = 0.995;

fn move_player_when_pressing_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Ball), With<Player>>,
) {
    for (mut transform, mut ball) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Detect movement directions based on key presses
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0; // Forward
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0; // Backward
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0; // Left
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0; // Right
        }

        // Normalize direction if there is movement
        if direction != Vec3::ZERO {
            direction = direction.normalize();
            ball.velocity += direction * MOVEMENT_SPEED; // Scale by movement speed
        }

        // Apply velocity to position
        let movement = ball.velocity; // Get the movement vector
        transform.translation += movement;

        // Calculate the amount of rotation based on the distance moved
        let distance = movement.length();
        let rotation_axis = Vec3::new(-movement.z, 0.0, movement.x).normalize(); // Rotation axis perpendicular to movement

        if distance > 0.0 {
            // Rotate the ball around the axis perpendicular to movement
            let rotation_angle = -distance / ball.radius; // The amount to rotate
            let rotation_quat = Quat::from_axis_angle(rotation_axis, rotation_angle);
            transform.rotation = rotation_quat * transform.rotation;
        }

        // Decrease velocity slowly each frame
        ball.velocity *= DAMPING_FACTOR; // Adjust the damping factor as needed
    }
}



// fn despawn_world(
//     mut commands: Commands,
//     query: Query<(Entity, Option<&Window>)>, // Use a query to find entities with the Window component
// ) {
//     // Find the window entity first
//     let window_entity = query.iter()
//         .filter(|(_, window)| window.is_some())
//         .map(|(entity, _)| entity)
//         .next();

//     // Despawn all entities except the window entity
//     for entity in query.iter() {
//         // Only despawn if the entity is not the window entity
//         if Some(entity.0) != window_entity {
//             commands.entity(entity.0).despawn_recursive();
//         }
//     }
// }

fn despawn_world(
    mut commands: Commands,
    window_query: Query<Entity, With<Window>>, // Query for the Window entity
    all_entities_query: Query<Entity>, // Query for all entities
) {
    // Get the window entity (assumes there is only one)
    let window_entity = window_query.iter().next();

    // Despawn all entities except the window entity
    for entity in all_entities_query.iter() {
        if Some(entity) != window_entity {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// fn check_player_out_of_bounds(
//     mut query: Query<(Entity, &mut Transform, &mut Ball), With<Player>>,
//     mut lives: Query<&mut Lives, With<Player>>,
//     mut lives_text: Query<&mut Text, With<LivesText>>,
//     mut game_state: ResMut<NextState<InGameState>>,
//     mut ball_velocities: Query<&mut Velocity, With<Player>>,
// ) {
//     for (entity, mut transform, mut ball) in query.iter_mut() {
//         let position = transform.translation;

//         // Check if the player is out of bounds
//         if position.x < MIN_X || position.x > MAX_X ||
//             position.y < MIN_Y || position.y > MAX_Y ||
//             position.z < MIN_Z || position.z > MAX_Z
//         {
//             println!("Player is out of bounds!");

//             // Decrease lives
//             for mut life in lives.iter_mut() {
//                 if life.lives == 0 {
//                     game_state.set(InGameState::GameOver);
//                     return;
//                 }
//                 life.lives -= 1;
//                 println!("Lives left: {}", life.lives);
//             }

//             // Update the UI text
//             for mut text in lives_text.iter_mut() {
//                 text.sections[0].value = format!("Lives: {}", lives.iter().next().unwrap().lives);
//             }

//             transform.translation = Vec3::new(0.0, 1.0, 0.0);
//             ball.velocity = Vec3::ZERO;

//             // Update the rigid body's position and velocity
//             if let Ok(mut rigid_body_velocity) = ball_velocities.get_mut(entity) {
//                 rigid_body_velocity.linvel = Vec3::ZERO; // Reset linear velocity
//                 rigid_body_velocity.angvel = Vec3::ZERO; // Optionally reset angular velocity
//             }
//         }
//     }
// }

fn handle_player_death(
    mut query: Query<(&mut Transform, &mut Ball), With<Player>>,
    mut lives: Query<&mut Lives, With<Player>>,
    mut lives_text: Query<&mut Text, With<LivesText>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    for (mut transform, mut ball) in query.iter_mut() {
        // Decrease lives
        for mut life in lives.iter_mut() {
            if life.lives == 0 {
                game_state.set(InGameState::GameOver);
                return;
            }
            life.lives -= 1;
        }

        // Update the UI text
        for mut text in lives_text.iter_mut() {
            text.sections[0].value = format!("Lives: {}", lives.iter().next().unwrap().lives);
        }

        transform.translation = Vec3::new(0.0, 1.0, 0.0);
        ball.velocity = Vec3::ZERO;

        game_state.set(InGameState::Playing);
    }
}

fn check_player_out_of_bounds(
    mut query: Query<(&mut Transform, &mut Ball), With<Player>>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    for (transform, _ball) in query.iter_mut() {
        let position = transform.translation;
        // Check if the player is out of bounds
        if position.x < MIN_X || position.x > MAX_X ||
            position.y < MIN_Y || position.y > MAX_Y ||
            position.z < MIN_Z || position.z > MAX_Z
        {
            next_state.set(InGameState::PlayerDied);
        }
    }
}

// Reset player's position to the center or a spawn point

#[derive(Component)]
struct ActivationTile {
    position: Vec3,
    activated: bool,
}


// MAIN MENU //

// TMP
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);


#[derive(Component)]
struct MainMenu;
fn setup_main_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
        ..default()
    };

    commands.spawn((Camera2dBundle::default(), MainMenu));

    commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::NONE),
            ..Default::default()
        }, MainMenu))
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(40.0),
                    height: Val::Percent(40.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                background_color: BackgroundColor::from(Color::srgba(0.4, 0.4, 0.4, 0.5)),
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Overball Game", text_style.clone(),
                ));
                parent.spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                }).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Start", text_style.clone(),
                    ));
                });
            });
        });
}

fn despawn_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenu>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn start_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut state: ResMut<NextState<AppState>>
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                // set game state to Game
                state.set(AppState::Game);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                text.sections[0].value = "Start".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}


fn play_gameover_sound(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
) {
    commands.spawn(AudioBundle {
        source: audio_assets.game_over_sound.clone(),
        settings: PlaybackSettings {
            volume: Volume::new(0.2),
            ..default()
        },
    });
}




// IGNORE
#[derive(Component)]
struct GameOverUI;

// TODO
fn setup_game_over_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
        color: Color::WHITE,
    };
    let button_style = TextStyle {
        font,
        font_size: 40.0,
        color: Color::BLACK,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                ..default()
            },
            GameOverUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Game Over!",
                text_style,
            ));
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.8, 0.8, 0.8, 1.0).into(),
                    ..default()
                },
            ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Restart",
                        button_style,
                    ));
                });
        });
}

// TODO
fn handle_game_over_input(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut in_game_state: ResMut<NextState<InGameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    game_over_ui: Query<Entity, With<GameOverUI>>,
    player_query: Query<Entity, With<Player>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Restart the game
                // Remove the game over UI
                for entity in game_over_ui.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Reset player health
                for player_entity in player_query.iter() {
                    commands.entity(player_entity).insert(Lives { lives: 3 });
                }

                in_game_state.set(InGameState::Playing);

                // You might want to reset other game elements here
                // For example, resetting the player's position, clearing enemies, etc.
            }
            Interaction::Hovered => {
                *color = Color::srgba(0.9, 0.9, 0.9, 1.0).into();
            }
            Interaction::None => {
                *color = Color::srgba(0.8, 0.8, 0.8, 1.0).into();
            }
        }
    }
}


// Pause Menu
fn pause_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<InGameState>>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            InGameState::Playing => {
                next_state.set(InGameState::Paused);
            }
            InGameState::Paused => {
                next_state.set(InGameState::Playing);
            }
            _ => {}
        }
    }
}

fn setup_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font,
        font_size: 50.0,
        color: Color::WHITE,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), // translucent background
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Paused\nPress ESC to resume", text_style,
            ));
        });
}

fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<Node>>, // Assuming the pause menu has Node component
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// Debug systems

// Debug system for GameState
// Debug system for GameState with state change detection
fn debug_game_state(
    state: ResMut<State<AppState>>,
    mut previous_state: Local<Option<AppState>>,
) {
    if let Some(prev) = previous_state.as_ref() {
        if *prev != **state {
            info!("GameState changed from {:?} to {:?}", prev, state);
            *previous_state = Some(state.clone());
        }
    } else {
        *previous_state = Some(state.clone());
    }
}

// Debug system for InGameState with state change detection
fn debug_in_game_state(
    state: ResMut<State<InGameState>>,
    mut previous_state: Local<Option<InGameState>>,
) {
    if let Some(prev) = previous_state.as_ref() {
        if *prev != **state {
            info!("InGameState changed from {:?} to {:?}", prev, state);
            *previous_state = Some(state.clone());
        }
    } else {
        *previous_state = Some(state.clone());
    }
}
/* A system that displays the events. */
fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.read() {
        info!("Received collision event: {:?}", collision_event);
    }

    for contact_force_event in contact_force_events.read() {
        info!("Received contact force event: {:?}", contact_force_event);
    }
}




