use bevy::prelude::*;
//use bevy::audio::Volume; TODO
use super::states::*;
use super::constants::*;
use super::resources::*;
use super::systems::despawn_player_and_map;

pub struct VictoryPlugin;

#[derive(Component)]
pub struct VictoryUI;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
            OnEnter(InGameState::Victory),
            setup_victory_ui,
            // TODO sound
        )
        .add_systems(
            Update,
            handle_victory_input.run_if(in_state(InGameState::Victory)),
        )
        .add_systems(OnExit(InGameState::Victory), despawn_player_and_map);
    }
}

fn setup_victory_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/montserrat.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 60.0,
        color: Color::WHITE,
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
            VictoryUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("You Win!", text_style.clone()));
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect {
                            left: Val::Px(10.0),
                            right: Val::Px(10.0),
                            top: Val::Px(5.0),
                            bottom: Val::Px(5.0),
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Restart", text_style.clone()));
                });
        });
}

fn handle_victory_input(
    mut commands: Commands,
    mut in_game_state: ResMut<NextState<InGameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    victory_ui: Query<Entity, With<VictoryUI>>,
    mut context: ResMut<GameContext>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Restart the game
                // Remove the game over UI
                *color = PRESSED_BUTTON.into();
                for entity in victory_ui.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Reset player lives
                context.lives = PLAYER_LIVES;
                in_game_state.set(InGameState::Reset);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
