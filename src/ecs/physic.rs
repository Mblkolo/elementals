use nphysics2d::world::World;
use specs::{System, Write};

pub struct Physic {
    pub world: World<f32>,
}

impl Default for Physic {
    fn default() -> Self {
        Physic {
            world: World::new(),
        }
    }
}

pub struct PhysicSystem;

impl<'a> System<'a> for PhysicSystem {
    type SystemData = (Write<'a, Physic>);

    fn run(&mut self, mut physic: Self::SystemData) {
        physic.world.set_timestep(1. / 50.);
        physic.world.step();
    }
}
