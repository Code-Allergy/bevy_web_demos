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

pub struct DefaultPluginsWithCustomWindow;

impl Plugin for DefaultPluginsWithCustomWindow {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            #[cfg(target_arch = "wasm32")]
            primary_window: Some(Window {
                canvas: Some("#game-window".into()),
                ..default()
            }),
            ..default()
        }));
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
