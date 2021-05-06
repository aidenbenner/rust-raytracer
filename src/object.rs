use std::str::MatchIndices;

use crate::material::*;
use crate::ray::*;
use crate::vec3::*;
use crate::*;

pub struct RayHit {
    pub col: Color,
    pub point: Point3,
    pub t: f64,
    pub normal: Vec3,
    pub front_face: bool,
}

pub trait Object: Sync + Send {
    fn material<'a>(&'a self) -> &'a Box<dyn Material>;
    fn hit(&self, ray: &Ray) -> Option<RayHit>;
}

pub struct Sphere {
    pub center: Point3,
    pub r: f64,
    pub color: Color,
    pub mat: Box<dyn Material>,
}

unsafe impl Sync for Sphere {}
unsafe impl Send for Sphere {}

impl Sphere {
    pub fn new(center: Point3, r: f64, color: Color, mat: Box<dyn Material>) -> Self {
        Sphere {
            center,
            r,
            color,
            mat,
        }
    }
}

const T_MIN: f64 = 0.0001;
const T_MAX: f64 = 100000000.;

impl Object for Sphere {
    fn hit(&self, ray: &Ray) -> Option<RayHit> {
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
        let delta = b * b - a * c * 4.;

        if delta < 0. {
            return None;
        }

        let t1 = (-b - delta.sqrt()) / (2.0 * a);
        let t2 = (-b + delta.sqrt()) / (2.0 * a);

        let valid_t = |t: f64| -> bool { T_MIN <= t && t <= T_MAX };

        let t = if valid_t(t1) {
            t1
        } else if valid_t(t2) {
            t2
        } else {
            return None;
        };

        let intersection_point = ray.cast(t);

        // let normals always points against the ray
        let normal_to_outside = (intersection_point - self.center).unit_vec();

        let (normal, front_face) = if normal_to_outside.dot(&dir) > 0. {
            // we are inside the object
            (-normal_to_outside, false)
        } else {
            (normal_to_outside, true)
        };

        let col = self.color;
        /*Color::of_rgb(
            0.5 * (1. + normal.x()),
            0.5 * (1. + normal.y()),
            0.5 * (1. + normal.z()),
        );*/

        let point = ray.cast(t);

        Some(RayHit {
            col,
            point,
            t,
            normal,
            front_face,
        })
    }

    fn material<'a>(&'a self) -> &'a Box<dyn Material> {
        &self.mat
    }
}
