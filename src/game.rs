use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct World {
    enemies: Vec<Enemy>,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Enemy {
    pub pos: Point,
    pub color: Color,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Color {
    Green = 1,
    Red = 2,
    Blue = 3,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> World {
        World {
            enemies: vec![Enemy {
                color: Color::Red,
                pos: Point { x: 10.0, y: 5.0 },
            }],
        }
    }

    pub fn step(&mut self) {
        for enemy in &mut self.enemies {
            enemy.pos.x -= 1.;
            enemy.pos.y -= 1.;
        }
    }
}

#[wasm_bindgen]
impl World {
    pub fn enemies_count(&self) -> usize {
        self.enemies.len()
    }

    pub fn enemy(&self, no: usize) -> Enemy {
        self.enemies[no]
    }
}
