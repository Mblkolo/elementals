use super::super::physic::{Physic, PhysicBody};
use super::{Enemy, Position};
use crate::ecs::EnemyKillEvent;
use na::Point2;
use nphysics2d::force_generator::{ForceGenerator, ForceGeneratorHandle};
use nphysics2d::math::{Force, ForceType};
use nphysics2d::object::{BodyPartHandle, BodySet};
use nphysics2d::solver::IntegrationParameters;
use specs::{Join, ReadStorage, System, Write};

type Point = Point2<f32>;

struct ExplosionForceGenerator {
    parts: Vec<BodyPartHandle>,
    positions: Vec<Point>,
}

impl Default for ExplosionForceGenerator {
    fn default() -> Self {
        return ExplosionForceGenerator {
            parts: Vec::new(),
            positions: Vec::new(),
        };
    }
}

impl ForceGenerator<f32> for ExplosionForceGenerator {
    fn apply(&mut self, _inter: &IntegrationParameters<f32>, bodies: &mut BodySet<f32>) -> bool {
        if self.positions.len() == 0 {
            return true;
        }

        let explosion_force = 10.0;

        for handle in &self.parts {
            if let Some(body) = bodies.body_mut(handle.0) {
                for pos in &self.positions {
                    let part = body.part(handle.1).unwrap();
                    let delta_pos = part.center_of_mass() - pos;
                    let force =
                        Force::linear(explosion_force * delta_pos.normalize() / delta_pos.norm());

                    body.apply_force(handle.1, &force, ForceType::VelocityChange, false);
                }
            }
        }

        true
    }
}

#[derive(Default)]
pub struct ExplosionSystem {
    generator_handle: Option<ForceGeneratorHandle>,
}

impl<'a> System<'a> for ExplosionSystem {
    type SystemData = (
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, PhysicBody>,
        ReadStorage<'a, EnemyKillEvent>,
        ReadStorage<'a, Position>,
        Write<'a, Physic>,
    );

    fn run(
        &mut self,
        (enemy_storage, body_storage, kill_event_storage, position_storage, mut physic): Self::SystemData,
    ) {
        if self.generator_handle.is_none() {
            let generator = ExplosionForceGenerator::default();
            let handle = physic.world.add_force_generator(generator);
            self.generator_handle = Some(handle);
        }

        let handle = self.generator_handle.unwrap();
        let generator_trait = physic.world.force_generator_mut(handle);
        let mut generator = generator_trait
            .downcast_mut::<ExplosionForceGenerator>()
            .unwrap();

        generator.positions = (&kill_event_storage, &position_storage)
            .join()
            .map(|(_, x)| x.point.clone())
            .collect::<Vec<_>>();

        generator.parts.clear();
        for (body, _e) in (&body_storage, &enemy_storage).join() {
            generator.parts.push(BodyPartHandle(body.handle, 0));
        }
    }
}
