use na::geometry::*;
use na::Real;
use na::Vector2;

type Point = Point2<f32>;
type Vector = Vector2<f32>;

pub fn get_cross_points_with_sphere(
    center: &Point,
    radius: f32,
    from: &Point,
    to: &Point,
) -> Vec<Point> {
    let translate = Vector::new(center.x, center.y);
    let origin_from = from - translate;
    let origin_to = to - translate;

    let result = get_cross_points(radius, &origin_from, &origin_to);

    let ray = origin_to - origin_from;
    result
        .iter()
        .filter(|p| (*p - origin_from).dot(&ray) > 0.)
        .map(|p| p + translate)
        .collect::<Vec<Point>>()
}

fn get_cross_points(radius: f32, from: &Point, to: &Point) -> Vec<Point> {
    const EPS: f32 = 0.000001;

    let a = from.y - to.y;
    let b = to.x - from.x;
    let c = from.x * to.y - to.x * from.y;

    let len = a * a + b * b;
    let x0 = -a * c / len;
    let y0 = -b * c / len;
    if c * c > radius * radius * len + EPS {
        return vec![];
    }
    if Real::abs(c * c - radius * radius * len) < EPS {
        return vec![Point::new(x0, y0)];
    }

    let d = radius * radius - c * c / len;
    let multiplicands = Real::sqrt(d / len);

    let point1 = Point::new(x0 + b * multiplicands, y0 - a * multiplicands);

    let point2 = Point::new(x0 - b * multiplicands, y0 + a * multiplicands);

    return vec![point1, point2];
}
