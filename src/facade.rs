use crate::ecs;
use na::geometry::Point2;
use pyro::{All, Read};
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Game {
    state: ecs::MainState,
    shoot_point: Point2<f32>,
    is_shooting: bool,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut state = ecs::MainState::new();
        state.init();

        Game {
            state: state,
            shoot_point: Point2::origin(),
            is_shooting: false,
        }
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
    pub fn set_shoot_point(&mut self, x: f32, y: f32) {
        self.shoot_point = Point2::new(x, y);
        self.update_game_shoot_point();
    }

    #[wasm_bindgen]
    pub fn set_shooting(&mut self, is_shooting: bool) {
        self.is_shooting = is_shooting;
        self.update_game_shoot_point();
    }

    fn update_game_shoot_point(&mut self) {
        let shoot_point = match self.is_shooting {
            true => Some(self.shoot_point.clone()),
            false => None,
        };

        self.state.set_shoot_point(shoot_point);
    }

    #[wasm_bindgen]
    pub fn set_shoot_force(&mut self, force: i32) {
        self.state.set_shoot_force(force);
    }

    #[wasm_bindgen]
    pub fn get_player_pos(&mut self) -> String {
        let player_entity = self
            .state
            .world
            .matcher::<All<(Read<ecs::Position>, Read<ecs::Player>)>>()
            .next();

        match player_entity {
            Some((pos, p)) => {
                let player = Some(Player {
                    x: pos.point.x,
                    y: pos.point.y,
                    radius: p.radius,
                });
                serde_json::to_string(&player).unwrap()
            }
            _ => "null".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn get_state(&mut self) -> String {
        use specs::Join;

        let player_storage = self.state.spec_world.read_storage::<ecs::Player>();
        let pos_storage = self.state.spec_world.read_storage::<ecs::Position>();
        let player = (&pos_storage, &player_storage).join().next();

        let scope = self
            .state
            .world
            .matcher::<All<(Read<ecs::Scope>,)>>()
            .next();

        let enemy_storage = self.state.spec_world.read_storage::<ecs::Enemy>();
        let color_storage = self.state.spec_world.read_storage::<ecs::Color>();

        let enemies = (&pos_storage, &enemy_storage, &color_storage)
            .join()
            .map(|(pos, enemy, color)| Enemy {
                x: pos.point.x,
                y: pos.point.y,
                radius: enemy.radius,
                is_white: color.is_white,
            })
            .collect::<Vec<_>>();

        let shots = self
            .state
            .world
            .matcher::<All<(Read<ecs::ShotTrace>,)>>()
            .map(|(decal,)| Shot {
                from_x: decal.from.x,
                from_y: decal.from.y,
                to_x: decal.to.x,
                to_y: decal.to.y,
            })
            .collect::<Vec<_>>();

        let state = GameState {
            player: match player {
                Some((pos, p)) => Some(Player {
                    x: pos.point.x,
                    y: pos.point.y,
                    radius: p.radius,
                }),
                _ => None,
            },
            scope: match scope {
                Some((s,)) => s.scope,
                _ => 0,
            },
            enemies: enemies,
            shots: shots,
        };

        serde_json::to_string(&state).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
struct GameState {
    player: Option<Player>,
    enemies: Vec<Enemy>,
    shots: Vec<Shot>,
    scope: u32,
}

#[derive(Serialize, Deserialize)]
struct Player {
    x: f32,
    y: f32,
    radius: f32,
}

#[derive(Serialize, Deserialize)]
struct Enemy {
    x: f32,
    y: f32,
    radius: f32,
    is_white: bool,
}

#[derive(Serialize, Deserialize)]
struct Shot {
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
}
