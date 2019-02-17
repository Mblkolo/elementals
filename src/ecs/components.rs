use na::Point2;
use specs::{Component, VecStorage};

type Point = Point2<f32>;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Explosion {
    pub position: Point,
}
