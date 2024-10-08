use std::time::Duration;
use bevy::app::{App, Startup};
use bevy::asset::Assets;
use bevy::audio::Volume;
use bevy::color::Color;
use bevy::color::palettes::basic::RED;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use bevy::render::mesh::{PlaneMeshBuilder, VertexAttributeValues};
use bevy_rapier3d::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;
use web_demos::{DefaultPluginsWithCustomWindow};
use web_demos::player::PlayerPlugin;

#[wasm_bindgen(js_name = sourceFile)]
pub fn source_file() -> String { include_str!("010-overball-game.rs").to_string() }
#[wasm_bindgen(js_name = demoName)]
pub fn demo_name() -> String { "A basic game from my childhood".to_string() }
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    start_game();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
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

#[wasm_bindgen(js_name = startGame)]
pub fn start_game() {
    let mut app = App::new();

    // Plugins
    app
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(DefaultPluginsWithCustomWindow);

    // States
    app
        .insert_state(GameState::Title)
        .init_state::<InGameState>();

    // Configure System Sets
    app.configure_sets(Update, (
        MainMenuSet::Update
            .run_if(in_state(GameState::Title)),
        GameplaySet::Update
            .run_if(in_state(GameState::Game))
            .run_if(in_state(InGameState::Playing)),
    ));

    // Main Menu
    app.configure_sets(OnEnter(GameState::Title), (
        MainMenuSet::Setup,
    ));

    app.configure_sets(OnExit(GameState::Title), (
        MainMenuSet::Cleanup,
    ));

    app.configure_sets(OnEnter(GameState::Game), (
        GameplaySet::Setup,
    ));

    // Add systems to sets
    app
        // Main Menu
        .add_systems(OnEnter(GameState::Title), (setup_main_menu_ui,).in_set(MainMenuSet::Setup))
        .add_systems(Update, (start_button_system,).in_set(MainMenuSet::Update))
        .add_systems(OnExit(GameState::Title), (despawn_main_menu,).in_set(MainMenuSet::Cleanup))

        // Game
        .add_systems(OnEnter(GameState::Game), (
            setup_game_ui,
            setup_game_camera,
            setup_map,
            setup_player
        ).in_set(GameplaySet::Setup))

        .add_systems(Update, (
            move_player_when_pressing_keys,
            check_player_out_of_bounds,
        ).in_set(GameplaySet::Update))

        // Game Over
        .add_systems(OnEnter(InGameState::PlayerDied), handle_player_death)
        .add_systems(Update, check_death_timer.run_if(in_state(InGameState::PlayerDied)));

    app.run();
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
    mut game_state: ResMut<NextState<GameState>>,
    mut gameplay_state: ResMut<NextState<InGameState>>,
    asset_server: Res<AssetServer>,
) {
    let ball_properties = BallProperties::default();
    // Player ball
    commands.spawn(PlayerBundle {
        lives: Lives { lives: 3 },
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
    });

    // TEMP
    commands.spawn(AudioBundle {
        source: asset_server.load("sounds/bg.mp3"),
        settings: PlaybackSettings {
            volume: Volume::new(0.2),
            ..default()
        },

        // ..default()
    });

    game_state.set(GameState::Game);
    gameplay_state.set(InGameState::Playing);
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

// Define the play area bounds
// Let the player clip off the playable area, but not fall off the world
const MIN_X: f32 = -20.0;
const MAX_X: f32 = 20.0;
const MIN_Y: f32 = -10.0;   // Keep Y > 0 to prevent falling out of the world
const MAX_Y: f32 = 10.0;
const MIN_Z: f32 = -20.0;
const MAX_Z: f32 = 20.0;

fn check_player_out_of_bounds(
    mut query: Query<(&mut Transform, &mut Ball), With<Player>>,
    mut lives: Query<&mut Lives, With<Player>>,
    mut lives_text: Query<&mut Text, With<LivesText>>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    for (mut transform, mut ball) in query.iter_mut() {
        let position = transform.translation;

        // Check if the player is out of bounds
        if position.x < MIN_X || position.x > MAX_X ||
            position.y < MIN_Y || position.y > MAX_Y ||
            position.z < MIN_Z || position.z > MAX_Z
        {
            println!("Player is out of bounds!");

            // Decrease lives
            for mut life in lives.iter_mut() {
                if life.lives == 0 {
                    game_state.set(InGameState::PlayerDied);
                    return;
                }
                life.lives -= 1;
                println!("Lives left: {}", life.lives);
            }

            // Update the UI text
            for mut text in lives_text.iter_mut() {
                text.sections[0].value = format!("Lives: {}", lives.iter().next().unwrap().lives);
            }


            reset_player_position(&mut transform, &mut ball);
        }
    }
}

// Reset player's position to the center or a spawn point
fn reset_player_position(transform: &mut Transform, ball: &mut Ball) {
    // Reset position to center of the play area
    transform.translation = Vec3::new(0.0, 1.0, 0.0);

    // Reset the velocity as well
    ball.velocity = Vec3::ZERO;

    println!("Player position reset.");
}

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
                parent.spawn((TextBundle::from_section(
                    "Overball Game", text_style.clone(),
                ), MainMenu));
                parent.spawn((ButtonBundle {
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
                }, MainMenu)).with_children(|parent| {
                    parent.spawn((TextBundle::from_section(
                        "Start", text_style.clone(),
                    ), MainMenu));
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
    mut state: ResMut<NextState<GameState>>
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                // set game state to Game
                state.set(GameState::Game);
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

#[derive(Component)]
struct DeathTimer {
    timer: Timer,
}

fn handle_player_death(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
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
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Game Over!",
                text_style,
            ));
        });

    commands.spawn(AudioBundle {
        source: asset_server.load("sounds/game_over.wav"),
        settings: PlaybackSettings {
            volume: Volume::new(0.2),
            ..default()
        },
    });

    // Spawn the death timer
    commands.spawn(DeathTimer {
        timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
    });
}

fn check_death_timer(
    time: Res<Time>,
    mut timer_query: Query<&mut DeathTimer>,
    mut in_game_state: ResMut<NextState<InGameState>>,
) {
    for mut death_timer in &mut timer_query {
        if death_timer.timer.tick(time.delta()).just_finished() {
            in_game_state.set(InGameState::GameOver);
        }
    }
}

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
    mut app_state: ResMut<NextState<GameState>>,
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

                // Reset game states
                app_state.set(GameState::Game);
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



