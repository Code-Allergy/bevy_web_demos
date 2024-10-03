use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

pub mod player;

pub fn get_window() -> Window {
    Window {
        canvas: Some("#demo_canvas".into()),
        ..default()
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
