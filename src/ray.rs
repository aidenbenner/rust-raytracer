use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Ray {
        Ray {
            origin,
            dir: dir.unit_vec(),
        }
    }

    pub fn cast(&self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }
}
