use super::retained_storage::{Retained, RetainedStorage};
use na::base::Vector2;
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyHandle, ColliderDesc, RigidBodyDesc};
use nphysics2d::world::World;
use specs::{Component, System, VecStorage, Write, WriteStorage};

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

pub struct RemoveBodyForDeletedEntitySystem;

impl<'a> System<'a> for RemoveBodyForDeletedEntitySystem {
    type SystemData = (Write<'a, Physic>, WriteStorage<'a, PhysicBody>);

    fn run(&mut self, (mut physic, mut body_storage): Self::SystemData) {
        let body_handles = body_storage
            .retained()
            .iter()
            .map(|b| b.handle)
            .collect::<Vec<_>>();

        physic.world.remove_bodies(&body_handles);
    }
}

#[derive(Clone)]
pub struct PhysicBody {
    pub handle: BodyHandle,
}

impl Component for PhysicBody {
    type Storage = RetainedStorage<Self, VecStorage<Self>>;
}

pub fn create_player_body(physic: &mut Physic) -> PhysicBody {
    let shape = ShapeHandle::new(Ball::new(0.25));
    let collider_desc = ColliderDesc::new(shape);

    let handle = RigidBodyDesc::new()
        .collider(&collider_desc)
        .translation(Vector2::new(20., 20.))
        .build(&mut physic.world)
        .handle();

    PhysicBody { handle }
}

pub fn create_enemy_body(physic: &mut Physic, radius: f32, position: Vector2<f32>) -> PhysicBody {
    let shape = ShapeHandle::new(Ball::new(radius));
    let collider_desc = ColliderDesc::new(shape).density(1.0);

    let handle = RigidBodyDesc::new()
        .collider(&collider_desc)
        .translation(position)
        .build(&mut physic.world)
        .handle();

    PhysicBody { handle }
}
