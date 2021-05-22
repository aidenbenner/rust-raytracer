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

pub enum Axis {
    XY,
    XZ,
    YZ,
}

pub struct FlipFace {
    pub obj : Arc<dyn Object>,
}

impl Object for FlipFace {
    fn hit(&self, ray: &Ray) -> Option<RayHit> {
        let hit = self.obj.hit(ray)?;
        if hit.front_face {
            return Some(hit);
        }
        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        self.obj.bounding_box()
    }
}

pub struct Rect {
    pub p0 : (f64, f64),
    pub p1 : (f64, f64),
    pub k : f64,
    pub axis : Axis,
    pub mat : Arc<dyn Material>,
}


impl Rect {
    pub fn new(p0: (f64, f64), p1: (f64, f64), k: f64, axis: Axis, mat: Arc<dyn Material>) -> Self { Self { p0, p1, k, axis, mat } }

    fn axis(&self) -> (usize, usize, usize) {
        match self.axis {
            Axis::XY => {
                // z, x, y
                (2, 0, 1)
            }
            Axis::XZ => {
                // y, x, z
                (1, 0, 2)
            }
            Axis::YZ => {
                // x, y, z
                (0, 1, 2)
            }
        }
    }
}

unsafe impl Send for Rect {}
unsafe impl Sync for Rect {}

impl Object for Rect {

    fn hit(&self, ray: &Ray) -> Option<RayHit> {
        let (perp, a0, a1)  = self.axis();
        // t at intersection of the plane
        let t = (self.k - ray.origin[perp]) / ray.dir[perp];
        if t < T_MIN || t > T_MAX {
            return None;
        }


        const EPS : f64 = 0.0001;
        let hit_0 = ray.origin[a0] + t * ray.dir[a0];
        let hit_1 = ray.origin[a1] + t * ray.dir[a1];

        if hit_0 - EPS < self.p0.0 || hit_0 + EPS > self.p0.1 || hit_1 - EPS < self.p1.0  || hit_1 + EPS > self.p1.1 {
            return None;
        }

        let mut normal = Vec3::empty();
        normal[perp] = 1.;

        let (normal, front_face) = if normal.dot(&ray.dir) > 0. {
            (-normal, true)
        } else {
            (normal, false)
        };

        let point = ray.cast(t);

        Some(RayHit { col: Color::of_rgb(1.,0.,0.), point, t, normal, front_face, mat: self.mat.clone()})
    }

    fn bounding_box(&self) -> Option<AABB> {
        let (perp, a0, a1)  = self.axis();
        let mut small = Vec3::empty();
        small[perp] = self.k - 0.0001;
        small[a0] = self.p0.0 - 0.001;
        small[a1] = self.p1.0 - 0.001;

        let mut big = Vec3::empty();
        big[perp] = self.k + 0.0001;
        big[a0] = self.p0.1 + 0.001;
        big[a1] = self.p1.1 + 0.001;

        Some(AABB::new(small, big))
    }
}
