use na::base::Vector2;
use na::geometry::Isometry2;
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyHandle, ColliderDesc, RigidBodyDesc};
use nphysics2d::world::World;
use specs::{Component, System, VecStorage, Write};

pub type PhysicWorld = World<f32>;

pub struct Physic {
    pub world: PhysicWorld,
}

impl Default for Physic {
    fn default() -> Self {
        Physic {
            world: PhysicWorld::new(),
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

#[derive(Component)]
#[storage(VecStorage)]
pub struct PhysicBody {
    pub handle: BodyHandle,
}

pub fn create_player_body(physic: &mut Physic) -> PhysicBody {
    let shape = ShapeHandle::new(Ball::new(1.5));
    let collider_desc =
        ColliderDesc::new(shape).position(Isometry2::new(Vector2::new(1.0, 2.0), 0.));

    let handle = RigidBodyDesc::new()
        .collider(&collider_desc)
        .build(&mut physic.world)
        .handle();

    PhysicBody { handle }
}
