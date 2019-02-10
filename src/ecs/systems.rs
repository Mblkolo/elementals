use super::physic::{Physic, PhysicBody};
use super::{Enemy, Player, Position, Velocity};
use nphysics2d::algebra::Velocity2;
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct PlayerMovementSystem;

impl<'a> System<'a> for PlayerMovementSystem {
    type SystemData = (
        Write<'a, Physic>,
        ReadStorage<'a, PhysicBody>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (mut physic, body_storage, player_storage, vel_storage): Self::SystemData) {
        for (b, _, vel) in (&body_storage, &player_storage, &vel_storage).join() {
            let body = physic.world.rigid_body_mut(b.handle).unwrap();
            body.set_velocity(Velocity2::linear(vel.velocity.x, vel.velocity.y));
        }
    }
}

pub struct EnemyMovementSystem;
impl<'a> System<'a> for EnemyMovementSystem {
    type SystemData = (
        Write<'a, Physic>,
        ReadStorage<'a, PhysicBody>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (mut physic, body_storage, enemy_storage, vel_storage): Self::SystemData) {
        for (b, _, vel) in (&body_storage, &enemy_storage, &vel_storage).join() {
            let body = physic.world.rigid_body_mut(b.handle).unwrap();
            body.set_velocity(Velocity2::linear(vel.velocity.x, vel.velocity.y));
        }
    }
}

pub struct PositionSyncSystem;

impl<'a> System<'a> for PositionSyncSystem {
    type SystemData = (
        Read<'a, Physic>,
        ReadStorage<'a, PhysicBody>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (physic, body_storage, mut pos_storage): Self::SystemData) {
        for (b, pos) in (&body_storage, &mut pos_storage).join() {
            let body = physic.world.rigid_body(b.handle).unwrap();
            let position = body.position();

            pos.point.x = position.translation.x;
            pos.point.y = position.translation.y;
        }
    }
}
