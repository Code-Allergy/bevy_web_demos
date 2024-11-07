use super::resources::GameContext;
use super::states::GameplaySet;
use super::states::InGameState;
use bevy::prelude::*;
use bevy::time::Timer;

// UI components
#[derive(Component)]
struct LivesText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
pub struct PopupMessage {
    pub timer: Timer,
}

impl PopupMessage {
    pub fn spawn(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        message: &str,
        duration: f32,
    ) {
        let font = asset_server.load("fonts/montserrat.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 28.0,
            color: Color::WHITE,
            ..default()
        };

        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Auto,
                    height: Val::Auto,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(message.to_string(), text_style.clone()),
                    PopupMessage {
                        timer: Timer::from_seconds(duration, TimerMode::Once),
                    },
                ));
            });
    }
}

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Playing), setup_game_ui)
            .add_systems(
                Update,
                (update_game_ui, update_popup_message)
                    .in_set(GameplaySet::Update)
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}

fn setup_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_context: Res<GameContext>,
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
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("Lives: {}", game_context.lives),
                    text_style.clone(),
                ),
                LivesText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    format!("Score: {}", game_context.score),
                    text_style.clone(),
                ),
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

fn update_popup_message(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Text, &mut PopupMessage)>,
) {
    for (entity, mut text, mut popup_message) in query.iter_mut() {
        popup_message.timer.tick(time.delta());

        if popup_message.timer.finished() {
            commands.entity(entity).despawn();
        } else {
            let alpha = popup_message.timer.fraction_remaining();
            text.sections[0].style.color.set_alpha(alpha);
        }
    }
}
