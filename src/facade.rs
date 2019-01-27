use ecs;
use pyro::{All, Read};
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
    pub fn set_player_direction(&mut self, x: f32, y: f32) {
        self.state.set_player_direction(&mut na::Vector2::new(x, y));
    }

    #[wasm_bindgen]
    pub fn get_state(&mut self) -> String {
        let player_pos = self
            .state
            .world
            .matcher::<All<(Read<ecs::Position>, Read<ecs::Player>)>>()
            .next()
            .unwrap()
            .0
            .point;

        let enemies = self
            .state
            .world
            .matcher::<All<(Read<ecs::Position>, Read<ecs::Enemy>)>>()
            .map(|(pos, enemy)| Enemy {
                x: pos.point.x,
                y: pos.point.y,
                radius: enemy.radius,
            })
            .collect::<Vec<_>>();

        let state = GameState {
            player: Player {
                x: player_pos.x,
                y: player_pos.y,
            },
            enemies: enemies,
        };

        serde_json::to_string(&state).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
struct GameState {
    player: Player,
    enemies: Vec<Enemy>,
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
