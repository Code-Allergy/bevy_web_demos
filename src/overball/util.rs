use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use super::states::*;


// Debug systems

// Debug system for GameState
// Debug system for GameState with state change detection
pub fn debug_game_state(state: ResMut<State<AppState>>, mut previous_state: Local<Option<AppState>>) {
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
pub fn debug_in_game_state(
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
/* A system that displays the collision events. */
pub fn display_events(
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
