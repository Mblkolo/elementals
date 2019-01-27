use ecs;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    state: ecs::MainState,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut state = ecs::MainState::new();
        state.init();

        Game { state: state }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.state.step();
    }

    #[wasm_bindgen]
    pub fn get_state(&mut self) -> String {
        let state = GameState {
            player: Player { x: 10., y: 10. },
        };

        serde_json::to_string(&state).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
struct GameState {
    player: Player,
}

#[derive(Serialize, Deserialize)]
struct Player {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
struct Enemy {
    x: f32,
    y: f32,
    radius: f32,
}
