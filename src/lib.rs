use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowTheme};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod overball;
pub mod player;

pub struct DefaultPluginsWithCustomWindow;

impl Plugin for DefaultPluginsWithCustomWindow {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            #[cfg(target_arch = "wasm32")]
            primary_window: Some(Window {
                canvas: Some("#game-window".into()),
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GameControlPlugin);
    }
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

#[wasm_bindgen(js_name = stopGame)]
pub fn stop_game() {
    log("Exiting game stop_game");
    SHOULD_EXIT.store(true, Ordering::SeqCst);
}

pub struct GameControlPlugin;

impl Plugin for GameControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, exit_system);
    }
}

fn exit_system(mut exit: EventWriter<AppExit>) {
    if SHOULD_EXIT.load(Ordering::SeqCst) {
        log("Exiting game exit_system");
        exit.send(AppExit::Success);
    }
}
