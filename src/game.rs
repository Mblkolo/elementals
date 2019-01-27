use rand::Rand;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct World {
    enemies: Vec<Enemy>,
    player: Player,
    latest_heat: Point,
    rand: Rand,
    game_over: bool,
    scope: i32,
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

use std::ptr;

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> World {
        let mut rand = Rand::new(0);
        let enemies = (0..10)
            .map(|_i| create_enemy(&mut rand))
            .collect::<Vec<_>>();

        World {
            player: Player {
                pos: Point { x: 25., y: 25. },
                speed: Vector { x: 0., y: 0. },
                gun: Gun {
                    ticks_to_fire: 0,
                    target: Point { x: 0., y: 0. },
                },
                firing: false,
            },
            latest_heat: Point { x: -10., y: -10. },
            game_over: false,
            scope: 0,
            rand: rand,
            enemies: enemies,
        }
    }

    //фиксированная частота обновления: 20мс
    pub fn step(&mut self) {
        if self.game_over {
            return;
        }

        if player_has_collision(&self.player.pos, &self.enemies) {
            self.game_over = true;
            return;
        }

        const INTERVAL: f32 = 0.02;
        const ENEMY_SPEED: f32 = 4.;
        const PLAYER_SPEED: f32 = 8.;

        self.player.pos.x += self.player.speed.x * INTERVAL * PLAYER_SPEED;
        self.player.pos.y += self.player.speed.y * INTERVAL * PLAYER_SPEED;

        if self.player.firing && self.player.gun.can_fire() {
            self.player.gun.fire();

            let target = self.player.gun.target;

            let may_be_pos = {
                let fired_enemy = get_fired_enemy(self.player.pos, target, &self.enemies);
                match fired_enemy {
                    None => None,
                    Some(enemy) => {
                        self.latest_heat = enemy.1;
                        self.enemies.iter().position(|e| ptr::eq(e, enemy.0))
                    }
                }
            };

            if let Some(pos) = may_be_pos {
                self.enemies.remove(pos);
                self.scope += 1;

                self.enemies.push(create_enemy(&mut self.rand));

                if self.rand.rand() % 3 == 0 {
                    self.enemies.push(create_enemy(&mut self.rand));
                }
            }
        } else {
            self.player.gun.wait();
        }

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

fn create_enemy(rand: &mut Rand) -> Enemy {
    let new_pos = if rand.rand() % 2 == 0 {
        Point {
            x: (25 + 24 * ((rand.rand() % 2) * 2 - 1)) as f32,
            y: (rand.rand() % 50) as f32,
        }
    } else {
        Point {
            x: (rand.rand() % 50) as f32,
            y: (25 + 24 * ((rand.rand() % 2) * 2 - 1)) as f32,
        }
    };

    Enemy {
        pos: new_pos,
        color: Color::Blue,
    }
}

fn has_collision(enemies: &Vec<Enemy>, point: &Point, exclude_id: usize) -> bool {
    for (i, enemy) in enemies.iter().enumerate() {
        if i == exclude_id {
            continue;
        }

        if has_circles_collision(&enemy.pos, point, 0.5 + 0.5) {
            return true;
        }
    }

    false
}

fn player_has_collision(player_pos: &Point, enemies: &Vec<Enemy>) -> bool {
    enemies
        .iter()
        .any(|e| has_circles_collision(&e.pos, player_pos, 0.5 + 0.1))
}

fn has_circles_collision(a: &Point, b: &Point, distance: f32) -> bool {
    let vector = a.sub(b);
    let len = abs(vector.x * vector.x) + abs(vector.y * vector.y);

    len < distance
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
        })
        .collect::<Vec<_>>();

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

    pub fn get_scope(&self) -> i32 {
        self.scope
    }
}
