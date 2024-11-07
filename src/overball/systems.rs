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
