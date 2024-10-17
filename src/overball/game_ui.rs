use bevy::prelude::*;
use super::states::{InGameState, AppState};
use super::constants::PLAYER_LIVES;
use super::resources::GameContext;
use super::states::GameplaySet;

// UI components
#[derive(Component)]
struct LivesText;

#[derive(Component)]
struct ScoreText;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup_game_ui)
            .add_systems(
                Update, update_game_ui
                    .in_set(GameplaySet::Update)
                    .run_if(in_state(InGameState::Playing)));
    }
}

fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            parent.spawn((
                TextBundle::from_section(format!("Lives: {}", PLAYER_LIVES), text_style.clone()),
                LivesText,
            ));
            parent.spawn((
                TextBundle::from_section("Score: 0".to_string(), text_style.clone()),
                ScoreText,
            ));
        });
}

fn update_game_ui(
    mut set: ParamSet<(
        Query<(&mut Text, &LivesText)>,
        Query<(&mut Text, &ScoreText)>,
    )>,
    game_context: Res<GameContext>,
) {
    if game_context.is_changed() {
        for (mut text, _) in set.p0().iter_mut() {
            text.sections[0].value = format!("Lives: {}", game_context.lives);
        }
        for (mut text, _) in set.p1().iter_mut() {
            text.sections[0].value = format!("Score: {}", game_context.score);
        }
    }
}
