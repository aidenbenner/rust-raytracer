use crate::object::*;
use crate::ray::*;
use crate::vec3::*;
use crate::*;

pub trait Material {
    fn scatter(&self, ray: &Ray, hit: &RayHit) -> Option<(Color, Ray)>;
}

pub struct Lambert {
    pub albedo: Color,
}

impl Material for Lambert {
    fn scatter(&self, ray: &Ray, hit: &RayHit) -> Option<(Color, Ray)> {
        let mut dir = Vec3::rand_in_hemisphere(&hit.normal);

        if dir.is_zero() {
            dir = hit.normal;
        }

        Some((self.albedo, Ray::new(hit.point, dir)))
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &RayHit) -> Option<(Color, Ray)> {
        let reflected = ray.dir.reflect(&hit.normal);

        let scattered = Ray::new(
            hit.point,
            reflected + Vec3::rand_in_unit_circle() * self.fuzz,
        );

        (scattered.dir.dot(&hit.normal) >= 0. || true).then(|| (self.albedo, scattered))
    }
}

pub struct Glass {
    pub refraction_index: f64,
}
impl Glass {
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1. - ref_idx) / (1. + ref_idx);
        r0 = r0 * r0;
        return r0 + (1. - r0) * (1. - cosine).powf(5.);
    }
}

impl Material for Glass {
    fn scatter(&self, ray: &Ray, hit: &RayHit) -> Option<(Color, Ray)> {
        let ref_indexes = if hit.front_face {
            (self.refraction_index, 1.)
        } else {
            (1., self.refraction_index)
        };

        let ref_ratio = ref_indexes.1 / ref_indexes.0;

        let incident = &ray.dir.unit_vec();
        let normal = hit.normal;

        let cos_theta = (-*incident).dot(&normal).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let mut rng = rand::thread_rng();
        let gen = Uniform::new_inclusive(0., 1.);

        let scattered = if ref_ratio * sin_theta > 1.
            || Self::reflectance(cos_theta, ref_ratio) > gen.sample(&mut rng)
        {
            //reflect
            ray.dir.reflect(&hit.normal)
        } else {
            // refract
            Vec3::refract(incident, &normal.unit_vec(), ref_indexes.0, ref_indexes.1)
        };

        Some((Color::white(), Ray::new(hit.point, scattered)))
    }
}
