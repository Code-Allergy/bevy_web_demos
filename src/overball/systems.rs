use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use super::components::*;
use super::resources::*;
use super::states::*;
use super::constants::*;

pub fn move_player_when_pressing_keys(
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
pub fn detect_ball_on_tile(
    ball_query: Query<&Transform, With<Player>>,
    mut tile_query: Query<(&Transform, &mut Tile, &mut Handle<StandardMaterial>)>,
    mut context: ResMut<GameContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(ball_transform) = ball_query.get_single() {
        let ball_position = ball_transform.translation;

        for (tile_transform, mut tile, mut material_handle) in tile_query.iter_mut() {
            let tile_position = tile_transform.translation;

            // Check if the ball is on top of the tile
            if (ball_position.x - tile_position.x).abs() < 0.5 &&
               (ball_position.z - tile_position.z).abs() < 0.5 {
                if !tile.activated {
                    tile.activated = true;
                    context.score += 1;
                    println!("Ball is on tile at position: {:?}. Score: {}", tile_position, context.score);

                    // Change the tile color to green
                    if let Some(material) = materials.get_mut(&*material_handle) {
                        material.base_color = Color::srgb(0.0, 1.0, 0.0);
                    }
                }
            }
        }
    }
}

pub fn detect_door_interaction(
    ball_query: Query<&Transform, With<Player>>,
    mut door_query: Query<(&mut Transform, &mut Door, &mut Collider)>,
    context: Res<GameContext>,
) {
    if let Ok(ball_transform) = ball_query.get_single() {
        let ball_position = ball_transform.translation;

        for (mut door_transform, mut door, mut collider) in door_query.iter_mut() {
            let door_position = door_transform.translation;

            // Check if the ball is close enough to the door to interact
            if (ball_position - door_position).length() < 1.0 {
                if context.score >= door.required_score && !door.is_open {
                    door.is_open = true;
                    door_transform.translation.y -= 1.0; // Move the door down into the floor
                    *collider = Collider::cuboid(0.5, 0.0, 0.5); // Disable the collider by setting its height to 0
                    println!("Door opened! Score: {}", context.score);
                }
            }
        }
    }
}

// pub fn handle_collisions(
//     mut collision_events: EventReader<CollisionEvent>,
//     mut ball_query: Query<(&mut Ball, &Transform), With<Player>>,
//     door_query: Query<Entity, With<Door>>,
// ) {
//     for collision_event in collision_events.iter() {
//         if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
//             if let Ok((mut ball, ball_transform)) = ball_query.get_mut(*entity1) {
//                 if door_query.get(*entity2).is_ok() {
//                     ball.velocity = Vec3::ZERO;
//                     println!("Ball collided with door at position: {:?}", ball_transform.translation);
//                 }
//             } else if let Ok((mut ball, ball_transform)) = ball_query.get_mut(*entity2) {
//                 if door_query.get(*entity1).is_ok() {
//                     ball.velocity = Vec3::ZERO;
//                     println!("Ball collided with door at position: {:?}", ball_transform.translation);
//                 }
//             }
//         }
//     }
// }

// pub fn update_score_text(
//     context: Res<GameContext>,
//     mut query: Query<&mut Text, With<ScoreText>>,
// ) {
//     for mut text in query.iter_mut() {
//         text.sections[0].value = format!("Score: {}", context.score);
//     }
// }

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

pub fn handle_game_over_input(
    mut commands: Commands,
    mut in_game_state: ResMut<NextState<InGameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    game_over_ui: Query<Entity, With<GameOverUI>>,
    mut context: ResMut<GameContext>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Restart the game
                // Remove the game over UI
                for entity in game_over_ui.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Reset player lives
                context.lives = PLAYER_LIVES;

                in_game_state.set(InGameState::Reset);

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
