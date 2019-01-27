pub use na::geometry::*;
use na::Vector2;
use pyro::*;

pub struct MainState {
    pub world: World,
    input: Input,
}

pub struct Input {
    player_direction: Vector2<f32>,
}

pub struct Player {
    max_speed: f32,
}
pub struct Enemy {
    radius: f32,
}

pub struct Position {
    pub point: Point2<f32>,
}

pub struct Velocity {
    velocity: Vector2<f32>,
}

impl MainState {
    pub fn new() -> MainState {
        let world = World::new();
        MainState {
            world,
            input: Input {
                player_direction: Vector2::new(0., 0.),
            },
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
    }

    pub fn step(self: &mut MainState) {
        update_player_velocity(&mut self.world, &self.input.player_direction);
        update_player_position(&mut self.world);
        update_enemies_position(&mut self.world);
    }

    pub fn set_player_direction(self: &mut MainState, direction: &mut Vector2<f32>) {
        if direction.norm() > 1.0 {
            direction.try_normalize_mut(0.01);
        }

        self.input.player_direction = direction.clone();
    }
}

fn update_player_velocity(world: &mut World, player_direction: &Vector2<f32>) {
    world
        .matcher::<All<(Read<Player>, Write<Velocity>)>>()
        .for_each(|(player, v)| {
            v.velocity = player_direction * player.max_speed;
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
                    && has_circles_collision(
                        &e.0.point,
                        &enemy.0.point,
                        e.2.radius + &enemy.2.radius,
                    )
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

fn has_circles_collision(a: &Point2<f32>, b: &Point2<f32>, minimum_distance: f32) -> bool {
    let distance = na::distance_squared(a, b);
    distance < minimum_distance * minimum_distance
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
            Enemy { radius: 0.5 },
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
            Enemy { radius: 0.5 },
            Position {
                point: Point2::new(0., 0.),
            },
            Velocity {
                velocity: Vector2::new(0.1, 0.2),
            },
        )));
        main_state.world.append_components(Some((
            Enemy { radius: 0.5 },
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
