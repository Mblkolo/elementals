use super::physic::{Physic, PhysicBody};
use super::{Enemy, Player, Position, Velocity};
use na::Point2;
use nphysics2d::algebra::Velocity2;
use nphysics2d::force_generator::{ForceGenerator, ForceGeneratorHandle};
use nphysics2d::math::{Force, ForceType};
use nphysics2d::object::{BodyPartHandle, BodySet};
use nphysics2d::solver::IntegrationParameters;
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct PlayerMovementSystem;

type Point = Point2<f32>;

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

//------
pub struct RadialForce {
    parts: Vec<BodyPartHandle>, // Body parts affected by the force generator.
    center: Point,
}

impl RadialForce {
    // Creates a new radial force generator.
    pub fn new(center: Point, parts: Vec<BodyPartHandle>) -> Self {
        RadialForce { parts, center }
    }

    /// Add a body part to be affected by this force generator.
    pub fn add_body_part(&mut self, body: BodyPartHandle) {
        self.parts.push(body)
    }
}

impl ForceGenerator<f32> for RadialForce {
    fn apply(&mut self, _: &IntegrationParameters<f32>, bodies: &mut BodySet<f32>) -> bool {
        for handle in &self.parts {
            if let Some(body) = bodies.body_mut(handle.0) {
                let part = body.part(handle.1).unwrap();

                let delta_pos = part.center_of_mass() - self.center;

                let force = Force::linear(delta_pos * -1.0);
                body.apply_force(handle.1, &force, ForceType::AccelerationChange, false);
            }
        }

        true
    }
}

pub struct EnemyForceMovementSystem {
    generator_handle: Option<ForceGeneratorHandle>,
}

impl Default for EnemyForceMovementSystem {
    fn default() -> Self {
        EnemyForceMovementSystem {
            generator_handle: None,
        }
    }
}

impl<'a> System<'a> for EnemyForceMovementSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, PhysicBody>,
        Write<'a, Physic>,
    );

    fn run(
        &mut self,
        (pos_storage, player_storage, enemy_storage, body_storage, mut physic): Self::SystemData,
    ) {
        if self.generator_handle.is_none() {
            let generator = RadialForce::new(Point::origin(), vec![]);
            let handle = physic.world.add_force_generator(generator);
            self.generator_handle = Some(handle);
        }

        let handle = self.generator_handle.unwrap();
        let generator_trait = physic.world.force_generator_mut(handle);
        let mut generator = generator_trait.downcast_mut::<RadialForce>().unwrap();

        let player = (&pos_storage, &player_storage).join().next();
        if let Some((p_pos, _)) = player {
            generator.center = p_pos.point;

            generator.parts.clear();
            for (body, _e) in (&body_storage, &enemy_storage).join() {
                generator.parts.push(BodyPartHandle(body.handle, 0));
            }
        }
    }
}
