use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use super::components::*;
use super::resources::*;
use super::states::*;
use super::constants::*;

pub fn move_player_when_pressing_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Ball), With<Player>>,
) {
    for (mut transform, mut ball) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // FORWARDS
        if keyboard_input.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
        // BACKWARDS
        if keyboard_input.pressed(KeyCode::KeyS) { direction.z += 1.0; }
        // LEFT
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        // RIGHT
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        // Normalize direction if there is movement
        if direction != Vec3::ZERO {
            direction = direction.normalize();
            ball.velocity += direction * MOVEMENT_SPEED * time.delta_seconds(); // Scale by movement speed
        }

        // Apply velocity to position
        let movement = ball.velocity; // Get the movement vector
        transform.translation += movement * time.delta_seconds();

        // Calculate the amount of rotation based on the distance moved
        let distance = movement.length() * time.delta_seconds();
        let rotation_axis = Vec3::new(-movement.z, 0.0, movement.x).normalize(); // Rotation axis perpendicular to movement

        if distance > 0.0 {
            // Rotate the ball around the axis perpendicular to movement
            let rotation_angle = -distance / ball.radius; // The amount to rotate
            let rotation_quat = Quat::from_axis_angle(rotation_axis, rotation_angle);
            transform.rotation = rotation_quat * transform.rotation;
        }

        // Decrease velocity slowly each frame
        ball.velocity *= DAMPING_FACTOR.powf(time.delta_seconds());
    }
}

pub fn detect_ball_on_tile(
    ball_query: Query<&Transform, With<Player>>,
    mut tile_query: Query<(&Transform, &mut Tile, &mut Handle<StandardMaterial>)>,
    mut context: ResMut<GameContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(ball_transform) = ball_query.get_single() {
        let ball_position = ball_transform.translation;

        for (tile_transform, mut tile, material_handle) in tile_query.iter_mut() {
            let tile_position = tile_transform.translation;

            // Check if the ball is on top of the tile
            if (ball_position.x - tile_position.x).abs() < 0.5 &&
               (ball_position.z - tile_position.z).abs() < 0.5 {
                if !tile.activated {
                    tile.activated = true;
                    context.score += 1;

                    // Change activated tile to be green
                    if let Some(material) = materials.get_mut(&*material_handle) {
                        material.base_color = Color::srgb(0.0, 1.0, 0.0);
                    }
                }
            }
        }
    }
}

pub fn check_player_out_of_bounds(
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

pub fn handle_player_death(
    mut query: Query<(&mut Transform, &mut Ball), With<Player>>,
    mut context: ResMut<GameContext>,
    mut game_state: ResMut<NextState<InGameState>>,
) {
    for (mut transform, mut ball) in query.iter_mut() {
        if context.lives == 0 {
            game_state.set(InGameState::GameOver);
        } else {
            // play respawn sound
            context.lives -= 1;
            transform.translation = Vec3::new(0.0, 1.0, 0.0);
            ball.velocity = Vec3::ZERO;
            game_state.set(InGameState::Playing);
        }

    }
}



pub fn despawn_player_and_map(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    player_camera_query: Query<Entity, With<PlayerCamera>>,
    map_query: Query<Entity, With<GameMap>>,
) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in player_camera_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in map_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Loading
pub fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bg_music = asset_server.load("sounds/bg.mp3");
    let door_thunk_sound = asset_server.load("sounds/door-thunk.wav");
    let door_opening_sound = asset_server.load("sounds/door-opening.mp3");
    let game_over_sound = asset_server.load("sounds/game_over.wav");
    let victory_sound = asset_server.load("sounds/victory.mp3");

    commands.insert_resource(AudioAssets {
        bg_music,
        game_over_sound,
        door_thunk_sound,
        door_opening_sound,
        victory_sound,
    });
}

pub fn check_assets_loaded(
    asset_server: Res<AssetServer>,
    audio_assets: Res<AudioAssets>,
    ball_asset: Res<BallAsset>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    if asset_server.get_load_state(&audio_assets.bg_music) == Some(bevy::asset::LoadState::Loaded)
        && asset_server.get_load_state(&audio_assets.game_over_sound) == Some(bevy::asset::LoadState::Loaded)
        && asset_server.get_load_state(&ball_asset.model) == Some(bevy::asset::LoadState::Loaded)
    {
        game_state.set(AppState::Title);
    }
}

pub fn load_ball_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load and store the handle
    let model = asset_server.load("models/Overball.glb#Scene0");

    commands.insert_resource(BallAsset { model });
}

pub fn check_winning_tile(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    winning_tile_query: Query<&Transform, With<WinningTile>>,
    mut timer_query: Query<(Entity, &mut WinningTileTimer)>,
    mut next_state: ResMut<NextState<InGameState>>,
    audio_assets: Res<AudioAssets>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;

        for winning_tile_transform in winning_tile_query.iter() {
            let tile_position = winning_tile_transform.translation;

            // Check if the player is on the winning tile
            if (player_position.x - tile_position.x).abs() < 2.5
                && (player_position.z - tile_position.z).abs() < 5.0
            {
                // If the timer already exists, update it
                if let Ok((entity, mut timer)) = timer_query.get_single_mut() {
                    timer.0.tick(time.delta());
                    if timer.0.finished() {
                        commands.spawn(AudioBundle {
                            source: audio_assets.victory_sound.clone(),
                            settings: PlaybackSettings {
                                volume: Volume::new(0.2),
                                ..default()
                            },
                        });

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

pub fn setup_background_music(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn(AudioBundle {
        source: audio_assets.bg_music.clone(),
        settings: PlaybackSettings {
            volume: Volume::new(0.2),
            mode: PlaybackMode::Loop,
            ..default()
        },
        
    });
}

pub fn clear_context(mut context: ResMut<GameContext>) {
    context.reset();
}
