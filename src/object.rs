use crate::ray::*;
use crate::vec3::*;
use crate::*;

pub trait Object {
    fn hit(&self, ray: &Ray) -> Option<(Color, f64)>;
}

pub struct Sphere {
    center: Point3,
    r: f64,
    color: Color,
}

impl Sphere {
    pub fn new(center: Point3, r: f64, color: Color) -> Self {
        Sphere { center, r, color }
    }
}

impl Object for Sphere {
    fn hit(&self, ray: &Ray) -> Option<(Color, f64)> {
        let origin = ray.origin;
        let dir = ray.dir;
        let center = self.center;
        // https://en.wikipedia.org/wiki/Line%E2%80%93sphere_intersection

        let a = dir.mag_squared();
        let diff = origin - center;
        let b = dir.dot(&diff) * 2.;
        let r_squared = self.r * self.r;
        let c = diff.mag_squared() - r_squared;

        // TODO can simplify
        let delta = b * b - diff.mag_squared() + r_squared;

        (delta >= 0.).then(|| {
            let t = -b * delta.sqrt() / (2.0 * a);

            let intersection_point = ray.cast(t);
            let normal = (intersection_point - self.center).unit_vec();

            let col = Color::of_rgb(
                0.5 * (1. + normal.x()),
                0.5 * (1. + normal.y()),
                0.5 * (1. + normal.z()),
            );

            (col, t)
        })
    }
}
