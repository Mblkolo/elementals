pub mod physic;

use crate::math;
use core::cmp::Ordering;
use na::geometry::*;
use na::Vector2;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use specs::{
    Builder, Component, Dispatcher, DispatcherBuilder, NullStorage, Read, ReadStorage, System,
    VecStorage, Write, WriteStorage,
};

type Point = Point2<f32>;
type Vector = Vector2<f32>;

pub struct MainState {
    pub spec_world: specs::World,
    dispatcher: Dispatcher<'static, 'static>,
    settings: Settings,
    rnd: SmallRng,
}

pub struct Input {
    player_direction: Vector,
    shoot_point: Option<Point>,
    shoot_force: i32,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            player_direction: Vector::zeros(),
            shoot_point: None,
            shoot_force: 0,
        }
    }
}

pub struct Settings {
    world_size: Point,
    fps: i32,
    gun_reload_ticks: i32,
    rnd: SmallRng,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            world_size: Point::new(10., 10.),
            fps: 50,
            gun_reload_ticks: 10,
            rnd: SmallRng::seed_from_u64(1),
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {
    max_speed: f32,
    pub radius: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Enemy {
    pub radius: f32,
    max_speed: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Gun {
    tick_to_reload: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Shot {
    from: Point,
    to: Point,
    force: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct DeadByTtl {
    ttl: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ShotTrace {
    pub from: Point,
    pub to: Point,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Color {
    pub is_white: bool,
    pub damage: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Spawner {
    pub tick_to_spawn: i32,
}

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct EnemyKillEvent;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Scope {
    pub scope: u32,
}

impl Enemy {
    fn default() -> Enemy {
        Enemy {
            radius: 0.5,
            max_speed: 4.,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    pub point: Point2<f32>,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    velocity: Vector,
}

impl MainState {
    pub fn new() -> MainState {
        let mut spec_world = specs::World::new();
        spec_world.add_resource(Settings {
            world_size: Point::new(50., 40.),
            fps: 50,
            gun_reload_ticks: 5,
            rnd: SmallRng::seed_from_u64(1),
        });

        spec_world.add_resource(physic::Physic::default());

        let mut dispatcher = DispatcherBuilder::new()
            .with(physic::PhysicSystem, "", &[])
            .with(RemoveByTtlSystem, "", &[])
            .with(UpdateTtlSystem, "", &[])
            .with(PlayerPositionSystem, "", &[])
            .with(PlayerVelocitySystem, "", &[])
            .with(ReturnPlayerToWarzoneSystem, "", &[])
            .with(EnemiesVelocitySystem, "", &[])
            .with(EnemiesPositionSystem, "", &[])
            .with(GunShotSystem, "", &[])
            .with(ShotSystem, "", &[])
            .with(RemoveOvercoloredEmenySystem, "", &[])
            .with(DamagePlayerSystem, "", &[])
            .with(ScopeSystem, "", &[])
            .with(SpawnEnemiesSystem, "", &[])
            .build();

        dispatcher.setup(&mut spec_world.res);

        spec_world.register::<Color>();
        spec_world.register::<Enemy>();
        spec_world.register::<Spawner>();
        spec_world.register::<Scope>();
        spec_world.register::<physic::PhysicBody>();

        MainState {
            spec_world: spec_world,
            settings: Settings {
                world_size: Point::new(50., 40.),
                fps: 50,
                gun_reload_ticks: 5,
                rnd: SmallRng::seed_from_u64(1),
            },
            dispatcher: dispatcher,
            rnd: SmallRng::seed_from_u64(0),
        }
    }

    pub fn init(self: &mut MainState) {
        let body_handle = {
            let mut phy = self.spec_world.write_resource::<physic::Physic>();
            physic::create_player_body(&mut phy)
        };

        self.spec_world
            .create_entity()
            .with(Player {
                max_speed: 6.,
                radius: 0.25,
            })
            .with(Position {
                point: Point2::new(
                    self.settings.world_size.x / 2.,
                    self.settings.world_size.y / 2.,
                ),
            })
            .with(Velocity {
                velocity: Vector2::new(0., 0.),
            })
            .with(Gun { tick_to_reload: 0 })
            .with(body_handle)
            .build();

        self.spec_world
            .create_entity()
            .with(Spawner { tick_to_spawn: 0 })
            .with(Scope { scope: 0 })
            .build();

        (0..10).for_each(|_| create_enemy(&mut self.spec_world, &self.settings, &mut self.rnd));
    }

    pub fn step(self: &mut MainState) {
        self.dispatcher.dispatch(&mut self.spec_world.res);
        self.spec_world.maintain();
    }

    pub fn set_player_direction(self: &mut MainState, direction: &mut Vector) {
        if direction.norm() > 1.0 {
            direction.try_normalize_mut(0.01);
        }

        self.spec_world.write_resource::<Input>().player_direction = direction.clone();
    }

    pub fn set_shoot_point(self: &mut MainState, shoot_point: Option<Point>) {
        self.spec_world.write_resource::<Input>().shoot_point = shoot_point;
    }

    pub fn set_shoot_force(self: &mut MainState, force: i32) {
        self.spec_world.write_resource::<Input>().shoot_force = force;
    }
}

struct ScopeSystem;

impl<'a> System<'a> for ScopeSystem {
    type SystemData = (
        WriteStorage<'a, Scope>,
        ReadStorage<'a, EnemyKillEvent>,
        specs::Entities<'a>,
    );

    fn run(&mut self, (mut scope_storage, kill_event_storage, entities): Self::SystemData) {
        use specs::Join;

        let scope = (&mut scope_storage).join().next();
        let kills = (&entities, &kill_event_storage)
            .join()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        if let Some(s) = scope {
            s.scope += kills.len() as u32;
        }

        for e in kills {
            entities.delete(e).unwrap();
        }
    }
}

struct DamagePlayerSystem;

impl<'a> System<'a> for DamagePlayerSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Enemy>,
        specs::Entities<'a>,
    );

    fn run(&mut self, (pos_storage, player_storage, enemy_storage, entities): Self::SystemData) {
        use specs::Join;

        let player = (&entities, &pos_storage, &player_storage).join().next();

        if let Some((entity, p_pos, p)) = player {
            let any_collision = (&pos_storage, &enemy_storage).join().any(|(e_pos, e)| {
                has_circles_collision(&e_pos.point, &p_pos.point, p.radius + e.radius)
            });

            if any_collision {
                entities.delete(entity).unwrap();
            }
        }
    }
}

struct SpawnEnemiesSystem;

impl<'a> System<'a> for SpawnEnemiesSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Scope>,
        WriteStorage<'a, Color>,
        specs::Entities<'a>,
        Write<'a, Settings>,
    );

    fn run(
        &mut self,
        (
            mut pos_storage,
            mut enemy_storage,
            mut vel_storage,
            scope_storage,
            mut color_storage,
            entities,
            mut settings,
        ): Self::SystemData,
    ) {
        use specs::Join;

        let maybe_scope = (&scope_storage).join().next();
        if let Some(scope) = maybe_scope {
            let target_count = scope.scope / 5 + 10;
            let current_count = enemy_storage.count() as u32;
            if current_count < target_count {
                let position = if settings.rnd.gen() {
                    Point::new(
                        settings.world_size.x * (settings.rnd.gen::<u32>() % 2) as f32,
                        settings.world_size.y * settings.rnd.gen::<f32>(),
                    )
                } else {
                    Point::new(
                        settings.world_size.x * settings.rnd.gen::<f32>(),
                        settings.world_size.y * (settings.rnd.gen::<u32>() % 2) as f32,
                    )
                };

                entities
                    .build_entity()
                    .with(Enemy::default(), &mut enemy_storage)
                    .with(Position { point: position }, &mut pos_storage)
                    .with(
                        Velocity {
                            velocity: Vector::zeros(),
                        },
                        &mut vel_storage,
                    )
                    .with(
                        Color {
                            is_white: settings.rnd.gen::<u32>() % 2 == 0,
                            damage: 0,
                        },
                        &mut color_storage,
                    )
                    .build();
            }
        }
    }
}

fn create_enemy<R: rand::Rng>(world: &mut specs::World, settings: &Settings, rnd: &mut R) {
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

    world
        .create_entity()
        .with(Enemy::default())
        .with(Position { point: position })
        .with(Velocity {
            velocity: Vector::zeros(),
        })
        .with(Color {
            is_white: rnd.gen::<u32>() % 2 == 0,
            damage: 0,
        })
        .build();
}

struct UpdateTtlSystem;

impl<'a> System<'a> for UpdateTtlSystem {
    type SystemData = (WriteStorage<'a, DeadByTtl>);

    fn run(&mut self, mut ttl_storage: Self::SystemData) {
        use specs::Join;

        (&mut ttl_storage).join().for_each(|d| d.ttl -= 1);
    }
}

struct RemoveByTtlSystem;

impl<'a> System<'a> for RemoveByTtlSystem {
    type SystemData = (WriteStorage<'a, DeadByTtl>, specs::Entities<'a>);

    fn run(&mut self, (ttl_storage, entities): Self::SystemData) {
        use specs::Join;

        (&entities, &ttl_storage)
            .join()
            .filter(|(_, d)| d.ttl <= 0)
            .for_each(|(e, _)| {
                entities.delete(e).unwrap();
            });
    }
}

struct ReturnPlayerToWarzoneSystem;

impl<'a> System<'a> for ReturnPlayerToWarzoneSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        specs::Read<'a, Settings>,
    );

    fn run(&mut self, (mut pos_storage, player_storage, settings): Self::SystemData) {
        use specs::Join;

        for (pos, pl) in (&mut pos_storage, &player_storage).join() {
            if pos.point.y - pl.radius < 0. {
                pos.point.y = pl.radius;
            }

            if pos.point.x - pl.radius < 0. {
                pos.point.x = pl.radius;
            }

            if pos.point.y + pl.radius > settings.world_size.y {
                pos.point.y = settings.world_size.y - pl.radius;
            }

            if pos.point.x + pl.radius > settings.world_size.x {
                pos.point.x = settings.world_size.x - pl.radius;
            }
        }
    }
}

struct PlayerVelocitySystem;
impl<'a> System<'a> for PlayerVelocitySystem {
    type SystemData = (
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Player>,
        specs::Read<'a, Input>,
        specs::Read<'a, Settings>,
    );

    fn run(&mut self, (mut vel_storage, player_storage, input, settings): Self::SystemData) {
        use specs::Join;

        for (vel, player) in (&mut vel_storage, &player_storage).join() {
            vel.velocity = input.player_direction * player.max_speed / settings.fps as f32;
        }
    }
}

struct PlayerPositionSystem;

impl<'a> System<'a> for PlayerPositionSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, (mut pos_storage, vel_storage, player_storage): Self::SystemData) {
        use specs::Join;

        for (p, v, _) in (&mut pos_storage, &vel_storage, &player_storage).join() {
            p.point += v.velocity;
        }
    }
}

struct EnemiesVelocitySystem;
impl<'a> System<'a> for EnemiesVelocitySystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Enemy>,
        WriteStorage<'a, Velocity>,
        Read<'a, Settings>,
    );

    fn run(
        &mut self,
        (pos_storage, player_storage, enemy_storage, mut vel_storage, settings): Self::SystemData,
    ) {
        use specs::Join;

        let player = (&pos_storage, &player_storage).join().next();

        if let Some((p_pos, _)) = player {
            for (e_pos, e_vel, e) in (&pos_storage, &mut vel_storage, &enemy_storage).join() {
                let direction = (p_pos.point - e_pos.point).try_normalize(0.001);
                e_vel.velocity = match direction {
                    Some(d) => d * e.max_speed / settings.fps as f32,
                    None => Vector::zeros(),
                }
            }
        }
    }
}

struct EnemiesPositionSystem;
impl<'a> System<'a> for EnemiesPositionSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (mut pos_storage, enemy_storage, vel_storage): Self::SystemData) {
        use specs::Join;
        let mut enemies = (&mut pos_storage, &vel_storage, &enemy_storage)
            .join()
            .collect::<Vec<_>>();

        for enemy_id in 0..enemies.len() {
            let maybe_pos = {
                let (e_pos, e_vel, enemy) = &enemies[enemy_id];
                let new_pos = e_pos.point + e_vel.velocity;

                let has_collision = enemies.iter().any(|(ae_pos, _, e)| {
                    std::ptr::eq(e, enemy) == false
                        && has_circles_collision(&ae_pos.point, &new_pos, e.radius + &enemy.radius)
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
}

struct GunShotSystem;
impl<'a> System<'a> for GunShotSystem {
    type SystemData = (
        specs::Entities<'a>,
        WriteStorage<'a, Gun>,
        WriteStorage<'a, Shot>,
        WriteStorage<'a, DeadByTtl>,
        ReadStorage<'a, Position>,
        specs::Read<'a, Input>,
        specs::Read<'a, Settings>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut gun_storage,
            mut shot_storage,
            mut ttl_storage,
            pos_storage,
            input,
            settings,
        ): Self::SystemData,
    ) {
        use specs::Join;

        if let Some(shoot_point) = input.shoot_point {
            let shots = (&mut gun_storage, &pos_storage)
                .join()
                .filter_map(|(gun, pos)| {
                    let shot = match gun.tick_to_reload {
                        0 => Some(Shot {
                            from: pos.point,
                            to: shoot_point,
                            force: input.shoot_force,
                        }),
                        _ => None,
                    };

                    gun.tick_to_reload = match gun.tick_to_reload {
                        x if x > 0 => x - 1,
                        _ => settings.gun_reload_ticks,
                    };

                    shot
                })
                .collect::<Vec<_>>();

            for shot in shots {
                entities
                    .build_entity()
                    .with(shot, &mut shot_storage)
                    .with(DeadByTtl { ttl: 0 }, &mut ttl_storage)
                    .build();
            }
        }
    }
}

struct ShotSystem;
impl<'a> System<'a> for ShotSystem {
    type SystemData = (
        specs::Entities<'a>,
        ReadStorage<'a, Shot>,
        WriteStorage<'a, Color>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Enemy>,
        WriteStorage<'a, DeadByTtl>,
        WriteStorage<'a, ShotTrace>,
    );

    fn run(
        &mut self,
        (
            entities,
            shot_storage,
            mut color_storage,
            pos_storage,
            enemy_storage,
            mut ttl_storage,
            mut trace_storage,
        ): Self::SystemData,
    ) {
        use specs::Join;

        let shots = (&shot_storage).join().collect::<Vec<_>>();

        let mut traces = Vec::new();
        for shot in shots {
            let enemies = (&enemy_storage, &pos_storage, &mut color_storage)
                .join()
                .collect::<Vec<_>>();

            let mut enemies_hits = vec![];
            for (enemy, enemy_pos, color) in enemies {
                let hit = get_enemy_hit_point(&shot, &enemy, &enemy_pos.point);
                if let Some(hit_pos) = hit {
                    enemies_hits.push((color, hit_pos));
                }
            }

            enemies_hits.sort_by(|(_, a), (_, b)| {
                compare_vector_lengths(&(a - shot.from), &(b - shot.from))
            });

            let enemy_hit = enemies_hits.first_mut();
            if let Some((color, _)) = enemy_hit {
                color.damage = shot.force;
            }

            traces.push(ShotTrace {
                from: shot.from.clone(),
                to: match enemy_hit {
                    Some((_, hit)) => *hit,
                    _ => shot.to.clone(),
                },
            })
        }

        //DeadByTtl { ttl: 5 },
        for trace in traces {
            entities
                .build_entity()
                .with(trace, &mut trace_storage)
                .with(DeadByTtl { ttl: 5 }, &mut ttl_storage)
                .build();
        }
    }
}

struct RemoveOvercoloredEmenySystem;
impl<'a> System<'a> for RemoveOvercoloredEmenySystem {
    type SystemData = (
        specs::Entities<'a>,
        ReadStorage<'a, Color>,
        ReadStorage<'a, Enemy>,
        WriteStorage<'a, EnemyKillEvent>,
    );

    fn run(
        &mut self,
        (entities, color_storage, enemy_storage, mut kill_event_storage): Self::SystemData,
    ) {
        use specs::Join;

        let enemies = (&entities, &enemy_storage, &color_storage)
            .join()
            .filter(|(_, _, color)| {
                color.is_white && color.damage > 0 || color.is_white == false && color.damage < 0
            })
            .map(|(entity, _, _)| entity)
            .collect::<Vec<_>>();

        for _ in 0..enemies.len() {
            entities
                .build_entity()
                .with(EnemyKillEvent {}, &mut kill_event_storage)
                .build();
        }

        for enemy in enemies {
            entities.delete(enemy).unwrap();
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_squared_test() {
        let distance = na::distance_squared(&Point2::new(0., 0.), &Point2::new(3., 4.));
        assert_eq!(25., distance);
    }

    #[test]
    fn normalize_test() {
        let vector = Vector2::new(0.0, 0.0);
        let vec2 = vector.try_normalize(0.01);

        assert!(vec2.is_none());
    }
}
