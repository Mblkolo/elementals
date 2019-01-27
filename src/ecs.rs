pub use na::geometry::*;
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
}

pub struct Settings {
    world_size: Point,
    fps: i32,
}

pub struct Player {
    max_speed: f32,
}
pub struct Enemy {
    pub radius: f32,
    max_speed: f32,
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
                player_direction: Vector2::new(0., 0.),
            },
            settings: Settings {
                world_size: Point::new(50., 20.),
                fps: 50,
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
        )));

        (0..10).for_each(|_| create_enemy(&mut self.world, &self.settings, &mut self.rnd));
    }

    pub fn step(self: &mut MainState) {
        update_player_velocity(
            &mut self.world,
            &self.input.player_direction,
            &self.settings,
        );
        update_player_position(&mut self.world);

        update_enemies_velocity(&mut self.world, &self.settings);
        update_enemies_position(&mut self.world);
    }

    pub fn set_player_direction(self: &mut MainState, direction: &mut Vector) {
        if direction.norm() > 1.0 {
            direction.try_normalize_mut(0.01);
        }

        self.input.player_direction = direction.clone();
    }
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
                point: Point2::new(0., 0.),
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
