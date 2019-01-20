use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct World {
    enemies: Vec<Enemy>,
    player: Player,
    latest_heat: Point,
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
    gun: Gun,
    firing: bool,
}

const RELOAD_TICKS: i32 = 5;
struct Gun {
    target: Point,
    ticks_to_fire: i32,
}

impl Gun {
    fn can_fire(&self) -> bool {
        self.ticks_to_fire == 0
    }

    fn fire(&mut self) {
        self.ticks_to_fire = RELOAD_TICKS;
    }

    fn wait(&mut self) {
        if self.ticks_to_fire > 0 {
            self.ticks_to_fire -= 1;
        }
    }
}

impl Point {
    fn sub(&self, point: &Point) -> Vector {
        Vector {
            x: self.x - point.x,
            y: self.y - point.y,
        }
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
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
                gun: Gun {
                    ticks_to_fire: 0,
                    target: Point { x: 0., y: 0. },
                },
                firing: false,
            },
            latest_heat: Point { x: 0., y: 0. },
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

        if self.player.firing && self.player.gun.can_fire() {
            self.player.gun.fire();

            let target = self.player.gun.target;
            //self.enemies.retain(|e| point_in_enemy(e, target) == false);
            let fired_enemy = get_fired_enemy(self.player.pos, target, &self.enemies);
            if let Some(enemy) = fired_enemy {
                // let pos = match self.enemies.iter().position(|x| *x == **enemy.0) {
                //     Some(x) => x,
                //     None => return None,
                // };

                //self.enemies.retain(|e| *enemy.0 == *e);
                self.latest_heat = enemy.1;
            }
        } else {
            self.player.gun.wait();
        }

        return;
        let target: Point = self.player.pos;
        for enemy_id in 0..self.enemies.len() {
            let new_pos = {
                let enemy = &self.enemies[enemy_id];

                let mut vector = target.sub(&enemy.pos);
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

    pub fn set_gan_target(&mut self, x: f32, y: f32) {
        self.player.gun.target = Point { x, y };
    }

    pub fn set_firing(&mut self, firing: bool) {
        self.player.firing = firing;
    }
}

fn point_in_enemy(enemy: &Enemy, point: &Point) -> bool {
    let distance = enemy.pos.sub(point);
    distance.x * distance.x + distance.y * distance.y < 0.5 * 0.5
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

use na::Real;
use std::cmp::Ordering;

fn get_fired_enemy(
    player_pos: Point,
    gun_target: Point,
    enemies: &Vec<Enemy>,
) -> Option<(&Enemy, Point)> {
    let mut fired_enemies = enemies
        .iter()
        .filter_map(|e| {
            let heat_point = get_enemy_heat_point(player_pos, gun_target, e);
            match heat_point {
                Some(point) => Some((e, point)),
                _ => None,
            }
        }).collect::<Vec<_>>();

    fired_enemies.sort_by(|a, b| {
        let da = a.1.sub(&player_pos);
        let db = b.1.sub(&player_pos);

        compare_len(&da, &db)
    });

    match fired_enemies.len() {
        0 => None,
        _ => Some(fired_enemies[0]),
    }
}

fn compare_len(da: &Vector, db: &Vector) -> Ordering {
    let a_len = da.x * da.x + da.y * da.y;
    let b_len = db.x * db.x + db.y * db.y;

    a_len.partial_cmp(&b_len).unwrap()
}

fn get_enemy_heat_point(player_pos: Point, gun_target: Point, enemy: &Enemy) -> Option<Point> {
    let mut cross_points = get_cross_points_with_sphere(enemy.pos, 0.5, player_pos, gun_target);

    //из всех точек выбираем самую ближайшую
    cross_points.sort_by(|a, b| {
        let da = a.sub(&player_pos);
        let db = b.sub(&player_pos);

        compare_len(&da, &db)
    });

    match cross_points.len() {
        0 => None,
        _ => Some(cross_points[0]),
    }
}

fn get_cross_points_with_sphere(
    center: Point,
    radius: f32,
    mut from: Point,
    mut to: Point,
) -> Vec<Point> {
    from.translate(-center.x, -center.y);
    to.translate(-center.x, -center.y);

    let mut result = get_cross_points(radius, from, to);
    for point in &mut result {
        point.translate(center.x, center.y);
    }

    result
}

fn get_cross_points(radius: f32, from: Point, to: Point) -> Vec<Point> {
    const EPS: f32 = 0.000001;

    let a = from.y - to.y;
    let b = to.x - from.x;
    let c = from.x * to.y - to.x * from.y;

    let len = a * a + b * b;
    let x0 = -a * c / len;
    let y0 = -b * c / len;
    if c * c > radius * radius * len + EPS {
        return vec![];
    }
    if abs(c * c - radius * radius * len) < EPS {
        return vec![Point { x: x0, y: y0 }];
    }

    let d = radius * radius - c * c / len;
    let multiplicands = Real::sqrt(d / len);

    let point1 = Point {
        x: x0 + b * multiplicands,
        y: y0 - a * multiplicands,
    };

    let point2 = Point {
        x: x0 - b * multiplicands,
        y: y0 + a * multiplicands,
    };

    return vec![point1, point2];
}

#[wasm_bindgen]
impl World {
    pub fn enemies_count(&self) -> usize {
        self.enemies.len()
    }

    pub fn enemy(&self, no: usize) -> Enemy {
        self.enemies[no]
    }

    pub fn latest_heat(&self) -> Point {
        self.latest_heat
    }
}
