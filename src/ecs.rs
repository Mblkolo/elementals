use crate::math;
use core::cmp::Ordering;
use na::geometry::*;
use na::Vector2;
use pyro::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

type Point = Point2<f32>;
type Vector = Vector2<f32>;

pub struct MainState {
    pub world: World,
    input: Input,
    settings: Settings,
    rnd: SmallRng,
}

pub struct Input {
    player_direction: Vector,
    shoot_point: Option<Point>,
    shoot_force: i32,
}

pub struct Settings {
    world_size: Point,
    fps: i32,
    gun_reload_ticks: i32,
    tick_to_spawn: i32,
}

pub struct Player {
    max_speed: f32,
}
pub struct Enemy {
    pub radius: f32,
    max_speed: f32,
}

pub struct Gun {
    tick_to_reload: i32,
}

pub struct Shot {
    from: Point,
    to: Point,
    force: i32,
}

pub struct DeadByTtl {
    ttl: i32,
}

pub struct ShotTrace {
    pub from: Point,
    pub to: Point,
}

pub struct Color {
    pub max: i32,
    pub current: i32,
}

pub struct Spawner {
    pub tick_to_spawn: i32,
}

impl Enemy {
    fn default() -> Enemy {
        Enemy {
            radius: 0.5,
            max_speed: 4.,
        }
    }
}

pub struct Position {
    pub point: Point2<f32>,
}

pub struct Velocity {
    velocity: Vector,
}

impl MainState {
    pub fn new() -> MainState {
        let world = World::new();
        MainState {
            world,
            input: Input {
                player_direction: Vector2::zeros(),
                shoot_point: None,
                shoot_force: 0,
            },
            settings: Settings {
                world_size: Point::new(50., 40.),
                fps: 50,
                gun_reload_ticks: 5,
                tick_to_spawn: 20,
            },
            rnd: SmallRng::seed_from_u64(0),
        }
    }

    pub fn init(self: &mut MainState) {
        self.world.append_components(Some((
            Player { max_speed: 6. },
            Position {
                point: Point2::new(5., 10.),
            },
            Velocity {
                velocity: Vector2::new(0., 0.),
            },
            Gun { tick_to_reload: 1 },
        )));

        self.world
            .append_components(Some((Spawner { tick_to_spawn: 0 },)));

        (0..10).for_each(|_| create_enemy(&mut self.world, &self.settings, &mut self.rnd));
    }

    pub fn step(self: &mut MainState) {
        update_ttl(&mut self.world);
        remove_by_ttl(&mut self.world);

        update_player_velocity(
            &mut self.world,
            &self.input.player_direction,
            &self.settings,
        );
        update_player_position(&mut self.world);

        update_enemies_velocity(&mut self.world, &self.settings);
        update_enemies_position(&mut self.world);

        shoot_from_gun(&mut self.world, &mut self.input, &self.settings);
        process_shots(&mut self.world);
        remove_overcolored_enemies(&mut self.world);

        spawn_enemies(&mut self.world, &self.settings, &mut self.rnd);
    }

    pub fn set_player_direction(self: &mut MainState, direction: &mut Vector) {
        if direction.norm() > 1.0 {
            direction.try_normalize_mut(0.01);
        }

        self.input.player_direction = direction.clone();
    }

    pub fn set_shoot_point(self: &mut MainState, shoot_point: Option<Point>) {
        self.input.shoot_point = shoot_point;
    }

    pub fn set_shoot_force(self: &mut MainState, force: i32) {
        self.input.shoot_force = force;
    }
}

fn spawn_enemies(world: &mut World, settings: &Settings, rnd: &mut SmallRng) {
    let mut count = 0;

    world
        .matcher::<All<(Write<Spawner>,)>>()
        .for_each(|(spawner,)| {
            if spawner.tick_to_spawn <= 0 {
                count += 1;
                spawner.tick_to_spawn = settings.tick_to_spawn
            } else {
                spawner.tick_to_spawn -= 1
            }
        });

    for _ in 0..count {
        create_enemy(world, settings, rnd);
    }
}

fn update_ttl(world: &mut World) {
    world
        .matcher::<All<(Write<DeadByTtl>,)>>()
        .for_each(|(d,)| d.ttl -= 1);
}

fn remove_by_ttl(world: &mut World) {
    let entities = world
        .matcher_with_entities::<All<(Read<DeadByTtl>,)>>()
        .filter(|(_, (d,))| d.ttl <= 0)
        .map(|(e, _)| e)
        .collect::<Vec<_>>();

    world.remove_entities(entities);
}

fn update_player_velocity(world: &mut World, player_direction: &Vector, settings: &Settings) {
    world
        .matcher::<All<(Read<Player>, Write<Velocity>)>>()
        .for_each(|(player, v)| {
            v.velocity = player_direction * player.max_speed / settings.fps as f32;
        });
}

fn update_player_position(world: &mut World) {
    world
        .matcher::<All<(Write<Position>, Read<Velocity>, Read<Player>)>>()
        .for_each(|(p, v, _)| {
            p.point += v.velocity;
        });
}

fn update_enemies_position(world: &mut World) {
    let mut enemies = world
        .matcher::<All<(Write<Position>, Read<Velocity>, Read<Enemy>)>>()
        .collect::<Vec<_>>();

    for enemy_id in 0..enemies.len() {
        let maybe_pos = {
            let enemy = &enemies[enemy_id];
            let new_pos = enemy.0.point + enemy.1.velocity;

            let has_collision = enemies.iter().any(|e| {
                std::ptr::eq(e, enemy) == false
                    && has_circles_collision(&e.0.point, &new_pos, e.2.radius + &enemy.2.radius)
            });

            match has_collision {
                true => None,
                false => Some(new_pos),
            }
        };

        if let Some(pos) = maybe_pos {
            enemies[enemy_id].0.point = pos;
        }
    }
}

fn update_enemies_velocity(world: &mut World, settings: &Settings) {
    let player = world
        .matcher::<All<(Read<Position>, Read<Player>)>>()
        .next();

    if let Some((p_pos, _)) = player {
        world
            .matcher::<All<(Read<Position>, Write<Velocity>, Read<Enemy>)>>()
            .for_each(|(e_pos, e_vel, e)| {
                let direction = (p_pos.point - e_pos.point).try_normalize(0.001);
                e_vel.velocity = match direction {
                    Some(d) => d * e.max_speed / settings.fps as f32,
                    None => Vector::zeros(),
                }
            });
    }
}

fn shoot_from_gun(world: &mut World, input: &Input, settings: &Settings) {
    if let Some(shoot_point) = input.shoot_point {
        let shots = world
            .matcher::<All<(Write<Gun>, Read<Position>)>>()
            .filter_map(|(gun, pos)| {
                gun.tick_to_reload = match gun.tick_to_reload {
                    x if x > 0 => x - 1,
                    _ => settings.gun_reload_ticks,
                };

                match gun.tick_to_reload {
                    0 => Some(Shot {
                        from: pos.point,
                        to: shoot_point,
                        force: input.shoot_force,
                    }),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        for shot in shots {
            world.append_components(Some((shot, DeadByTtl { ttl: 0 })));
        }
    }
}

fn process_shots(world: &mut World) {
    let shots = world.matcher::<All<(Read<Shot>,)>>().collect::<Vec<_>>();

    let mut shot_decals = Vec::new();
    for (shot,) in shots {
        let enemies = world
            .matcher::<All<(Read<Enemy>, Read<Position>, Write<Color>)>>()
            .collect::<Vec<_>>();

        let mut enemies_hits = vec![];
        for (enemy, enemy_pos, color) in enemies {
            let hit = get_enemy_hit_point(&shot, &enemy, &enemy_pos.point);
            if let Some(hit_pos) = hit {
                enemies_hits.push((color, hit_pos));
            }
        }

        enemies_hits
            .sort_by(|(_, a), (_, b)| compare_vector_lengths(&(a - shot.from), &(b - shot.from)));

        let enemy_hit = enemies_hits.first_mut();
        if let Some((color, _)) = enemy_hit {
            color.current += shot.force;
        }

        shot_decals.push((
            ShotTrace {
                from: shot.from.clone(),
                to: match enemy_hit {
                    Some((_, hit)) => *hit,
                    _ => shot.to.clone(),
                },
            },
            DeadByTtl { ttl: 5 },
        ))
    }

    world.append_components(shot_decals);
}

fn remove_overcolored_enemies(world: &mut World) {
    let enemies = world
        .matcher_with_entities::<All<(Read<Enemy>, Read<Color>)>>()
        .filter(|(_, (_, color))| color.current < 0 || color.current > color.max)
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    world.remove_entities(enemies);
}

fn get_enemy_hit_point(shot: &Shot, enemy: &Enemy, enemy_pos: &Point) -> Option<Point> {
    let mut cross_points =
        math::get_cross_points_with_sphere(&enemy_pos, enemy.radius, &shot.from, &shot.to);

    //из всех точек выбираем самую ближайшую
    cross_points.sort_by(|a, b| compare_vector_lengths(&(a - shot.from), &(b - shot.from)));

    match cross_points.len() {
        0 => None,
        _ => Some(cross_points[0]),
    }
}

fn compare_vector_lengths(a: &Vector, &b: &Vector) -> Ordering {
    let a_len = a.norm_squared();
    let b_len = b.norm_squared();

    a_len.partial_cmp(&b_len).unwrap()
}

fn has_circles_collision(a: &Point2<f32>, b: &Point2<f32>, minimum_distance: f32) -> bool {
    let distance = na::distance_squared(a, b);
    distance < minimum_distance * minimum_distance
}

fn create_enemy<R: rand::Rng>(world: &mut World, settings: &Settings, rnd: &mut R) {
    let position = if rnd.gen() {
        Point::new(
            settings.world_size.x * (rnd.gen::<u32>() % 2) as f32,
            settings.world_size.y * rnd.gen::<f32>(),
        )
    } else {
        Point::new(
            settings.world_size.x * rnd.gen::<f32>(),
            settings.world_size.y * (rnd.gen::<u32>() % 2) as f32,
        )
    };

    world.append_components(Some((
        Enemy::default(),
        Position { point: position },
        Velocity {
            velocity: Vector::zeros(),
        },
        Color {
            current: (rnd.gen::<u32>() % 6) as i32,
            max: 5,
        },
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_player_position_test() {
        let mut main_state = MainState::new();
        main_state.world.append_components(Some((
            Player { max_speed: 10. },
            Position {
                point: Point2::origin(),
            },
            Velocity {
                velocity: Vector2::new(1., 2.),
            },
        )));

        update_player_position(&mut main_state.world);

        let entities = main_state.world.entities().collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);

        let position = main_state.world.get_component::<Position>(entities[0]);
        assert!(position.is_some());
        assert_eq!(position.unwrap().point, Point2::new(1., 2.));
    }

    #[test]
    fn distance_squared_test() {
        let distance = na::distance_squared(&Point2::new(0., 0.), &Point2::new(3., 4.));
        assert_eq!(25., distance);
    }

    #[test]
    fn update_enemies_position_test() {
        let mut main_state = MainState::new();
        main_state.world.append_components(Some((
            Enemy::default(),
            Position {
                point: Point2::new(0., 0.),
            },
            Velocity {
                velocity: Vector2::new(1., 2.),
            },
        )));

        update_enemies_position(&mut main_state.world);

        let entities = main_state.world.entities().collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);

        let position = main_state.world.get_component::<Position>(entities[0]);
        assert!(position.is_some());
        assert_eq!(position.unwrap().point, Point2::new(1., 2.));
    }

    #[test]
    fn update_enemies_position_with_collision_enemies_test() {
        let mut main_state = MainState::new();
        main_state.world.append_components(Some((
            Enemy::default(),
            Position {
                point: Point2::new(0., 0.),
            },
            Velocity {
                velocity: Vector2::new(0.1, 0.2),
            },
        )));
        main_state.world.append_components(Some((
            Enemy::default(),
            Position {
                point: Point2::new(0., 0.),
            },
            Velocity {
                velocity: Vector2::new(0.1, 0.2),
            },
        )));

        update_enemies_position(&mut main_state.world);

        let entities = main_state.world.entities().collect::<Vec<_>>();
        assert_eq!(entities.len(), 2);

        let position = main_state.world.get_component::<Position>(entities[0]);
        assert_eq!(position.unwrap().point, Point2::new(0., 0.));

        let position = main_state.world.get_component::<Position>(entities[1]);
        assert_eq!(position.unwrap().point, Point2::new(0., 0.));
    }

    #[test]
    fn normalize_test() {
        let vector = Vector2::new(0.0, 0.0);
        let vec2 = vector.try_normalize(0.01);

        assert!(vec2.is_none());
    }
}
