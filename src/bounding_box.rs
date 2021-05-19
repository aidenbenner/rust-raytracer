use crate::{object::Object};


use crate::vec3::*;


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AABB {
    pub start : Vec3,
    pub end : Vec3,
}

impl PartialOrd for AABB {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.start;
        let b = other.start;

        a.x().partial_cmp(&b.x())
    }
}


impl AABB {
    pub fn new(start: Vec3, end: Vec3) -> Self { Self { start, end } }

    pub fn hit(&self, r: &crate::ray::Ray, mut t_min : f64, mut t_max : f64) -> bool {
        for a in 0..3 {
            let inv_d = 1. / r.dir[a];
            let mut t0 = (self.start[a] - r.origin[a]) * inv_d;
            let mut t1 = (self.end[a] - r.origin[a]) * inv_d;
            if inv_d < 0. {
                std::mem::swap(&mut t0, &mut t1)
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn combine(&self, other : &AABB) -> Self {
        let start = vec3![
            self.start[0].min(other.start[0]),
            self.start[1].min(other.start[1]),
            self.start[2].min(other.start[2])
        ];

        let end = vec3![
            self.end[0].max(other.end[0]),
            self.end[1].max(other.end[1]),
            self.end[2].max(other.end[2])
        ];

        Self::new(start,end)
    }
}
