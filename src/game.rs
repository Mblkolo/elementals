use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct World {
    enemies: Vec<Enemy>,
    player: Player,
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

struct Player {
    pos: Point,
    speed: Vector,
}

impl Point {
    fn sub(&self, point: &Point) -> Vector {
        Vector {
            x: self.x - point.x,
            y: self.y - point.y,
        }
    }
}

struct Vector {
    pub x: f32,
    pub y: f32,
}

fn abs(value: f32) -> f32 {
    if value > 0. {
        return value;
    }

    -value
}

impl Vector {
    fn normalize(&mut self) {
        let x = abs(self.x);
        let y = abs(self.y);
        let len = x + y;
        self.x /= len;
        self.y /= len;
    }
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> World {
        World {
            player: Player {
                pos: Point { x: 0., y: 0. },
                speed: Vector { x: 0., y: 0. },
            },
            enemies: (0..10)
                .map(|i| Enemy {
                    color: Color::Red,
                    pos: Point {
                        x: 10.0 + i as f32,
                        y: 5.0 + (2 * i) as f32,
                    },
                }).collect(),
        }
    }

    //фиксированная частота обновления: 20мс
    pub fn step(&mut self) {
        const INTERVAL: f32 = 0.02;
        const ENEMY_SPEED: f32 = 4.;
        const PLAYER_SPEED: f32 = 5.;

        self.player.pos.x += self.player.speed.x * INTERVAL * PLAYER_SPEED;
        self.player.pos.y += self.player.speed.y * INTERVAL * PLAYER_SPEED;

        let TARGET: Point = self.player.pos;
        for enemy_id in 0..self.enemies.len() {
            let new_pos = {
                let enemy = &self.enemies[enemy_id];

                let mut vector = TARGET.sub(&enemy.pos);
                vector.normalize();

                Point {
                    x: enemy.pos.x + vector.x * INTERVAL * ENEMY_SPEED,
                    y: enemy.pos.y + vector.y * INTERVAL * ENEMY_SPEED,
                }
            };

            if has_collision(&self.enemies, &new_pos, enemy_id) == false {
                let mut enemy = &mut self.enemies[enemy_id];
                enemy.pos = new_pos;
            }
        }
    }

    pub fn set_player_pos(&mut self, x: f32, y: f32) {
        self.player.pos = Point { x: x, y: y }
    }

    pub fn get_player_pos(&mut self) -> Point {
        self.player.pos
    }

    pub fn set_player_speed(&mut self, x: f32, y: f32) {
        self.player.speed = Vector { x, y };
    }
}

fn has_collision(enemies: &Vec<Enemy>, point: &Point, exclude_id: usize) -> bool {
    for (i, enemy) in enemies.iter().enumerate() {
        if i == exclude_id {
            continue;
        }
        let vector = enemy.pos.sub(point);
        let len = abs(vector.x * vector.x) + abs(vector.y * vector.y);
        if len < 1.0 * 1.0 {
            return true;
        }
    }

    false
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
