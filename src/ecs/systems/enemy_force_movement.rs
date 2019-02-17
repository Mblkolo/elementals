use super::super::physic::{Physic, PhysicBody};
use super::{Enemy, Player, Position};
use na::Point2;
use nphysics2d::force_generator::{ForceGenerator, ForceGeneratorHandle};
use nphysics2d::math::{Force, ForceType};
use nphysics2d::object::{BodyPartHandle, BodySet};
use nphysics2d::solver::IntegrationParameters;
use specs::{Join, ReadStorage, System, Write};

type Point = Point2<f32>;

pub struct RadialForce {
    parts: Vec<BodyPartHandle>, // Body parts affected by the force generator.
    center: Point,
}

impl RadialForce {
    // Creates a new radial force generator.
    pub fn new(center: Point, parts: Vec<BodyPartHandle>) -> Self {
        RadialForce { parts, center }
    }

    // Add a body part to be affected by this force generator.
    // pub fn add_body_part(&mut self, body: BodyPartHandle) {
    //     self.parts.push(body)
    // }
}

impl ForceGenerator<f32> for RadialForce {
    fn apply(&mut self, inter: &IntegrationParameters<f32>, bodies: &mut BodySet<f32>) -> bool {
        let low_friction = 5.0;
        let high_friction = 25.0;
        let speed_force = 20.0;
        let max_speed = 5.0;

        for handle in &self.parts {
            if let Some(body) = bodies.body_mut(handle.0) {
                let part = body.part(handle.1).unwrap();

                let velocity = part.velocity().linear;
                let speed = velocity.norm();
                if speed > 0.0001 {
                    let friction = if speed > max_speed {
                        high_friction
                    } else {
                        low_friction
                    };

                    let friction_force = -friction * velocity.normalize();
                    if (friction_force * inter.dt).norm() > speed {
                        //Полная остановка
                        let force = Force::linear(-velocity);
                        body.apply_force(handle.1, &force, ForceType::VelocityChange, false);
                    } else {
                        let force = Force::linear(friction_force);
                        body.apply_force(handle.1, &force, ForceType::AccelerationChange, false);
                    }
                }

                let part = body.part(handle.1).unwrap();
                let delta_pos = self.center - part.center_of_mass();
                let force = Force::linear(delta_pos.normalize() * speed_force);
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
