use bevy::prelude::*;
use super::states::InGameState;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(InGameState::Paused), setup_pause_menu)
        .add_systems(OnExit(InGameState::Paused), despawn_pause_menu)
        .add_systems(Update, pause_game_input);
    }
}


fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                "Paused\nPress ESC to resume",
                text_style,
            ));
        });
}

pub fn pause_game_input(
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

fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<Node>>, // Assuming the pause menu has Node component
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
