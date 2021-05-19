use std::{rc::Rc, str::MatchIndices};
use std::sync::Arc;

use crate::material::*;
use crate::ray::*;
use crate::vec3::*;
use crate::bounding_box::*;
use crate::*;

pub struct RayHit {
    pub col: Color,
    pub point: Point3,
    pub t: f64,
    pub normal: Vec3,
    pub front_face: bool,
    pub mat: Arc<dyn Material>,
}

pub trait Object: Sync + Send {
    fn hit(&self, ray: &Ray) -> Option<RayHit>;
    fn bounding_box(&self) -> Option<AABB>;
}

pub struct ObjectGroup {
    objs : Vec<Box<dyn Object>>,
    bb : AABB,
}

impl ObjectGroup {
    fn new(objs: Vec<Box<dyn Object>>) -> Self {
        let bb = objs.iter().map(|x| x.bounding_box())
                        .reduce(|a , b| {
                            let a = a?;
                            b.map(|b| {
                                a.combine(&b)
                            })
                        }).unwrap().unwrap();


        Self { objs, bb }
    }

    pub fn create_hierarchy(mut objs : Vec<Box<dyn Object>>) -> Self {
        let mut rng = rand::thread_rng();
        if objs.len() <= 2 {
            return Self::new(objs);
        }

        objs.sort_by_cached_key(|x| {
            (x.bounding_box().unwrap().start[rng.gen_range(0..3)] * 100000.) as i64
        });

        let mut lhs = Vec::with_capacity(objs.len() / 2);
        let mut rhs = Vec::with_capacity(objs.len() / 2);

        let N = objs.len();
        for (i, obj) in objs.into_iter().enumerate() {
            if i < N / 2 {
                lhs.push(obj)
            } else {
                rhs.push(obj)
            }
        }

        let lhs = Self::create_hierarchy(lhs);
        let rhs = Self::create_hierarchy(rhs);


        return Self::new(vec![Box::new(lhs), Box::new(rhs)]);
    }


}

impl Object for ObjectGroup {
    fn hit(&self, ray: &Ray) -> Option<RayHit> {
        if !self.bb.hit(ray, T_MIN, T_MAX) {
            return None;
        }


        self.objs.iter().fold(None, |acc, obj|{
                if let Some(hit) = obj.hit(&ray) {
                    match &acc {
                        Some(acc) => {
                            if hit.t < acc.t {
                                return Some(hit)
                            }
                        }
                        None => {
                            return Some(hit)
                        }
                    }
                }
                acc
            }
        )
    }


    fn bounding_box(&self) -> Option<AABB> {
        self.objs.iter().map(|x| x.bounding_box())
                        .reduce(|a , b| {
                            let a = a?;
                            b.map(|b| {
                                a.combine(&b)
                            })
                        })?
    }
}


pub struct Sphere {
    pub center: Point3,
    pub r: f64,
    pub color: Color,
    pub mat: Arc<dyn Material>,
}

unsafe impl Sync for Sphere {}
unsafe impl Send for Sphere {}

impl Sphere {
    pub fn new(center: Point3, r: f64, color: Color, mat: Arc<dyn Material>) -> Self {
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
        if !self.bounding_box().unwrap().hit(ray, T_MIN, T_MAX) {
            return None;
        }

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
        let point = ray.cast(t);

        Some(RayHit {
            col,
            point,
            t,
            normal,
            front_face,
            mat:self.mat.clone()
        })
    }

    fn bounding_box(&self) -> Option<AABB> {
        let rvec = vec3![self.r, self.r, self.r];
        Some(AABB::new(
            self.center - rvec,
            self.center + rvec,
        ))
    }
}
