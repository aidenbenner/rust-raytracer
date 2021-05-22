use anyhow::{anyhow, Result};
use material::{DiffuseLight, Glass, Lambert, Material, Metal};
use object::{Axis, FlipFace, ObjectGroup, RayHit, Rect};
use rayon::prelude::*;
use vec3::Point3;

use std::{fs::File, rc::Rc, sync::Arc};
use std::{io::Write, sync::atomic::AtomicI64};

mod material;
mod object;
mod ray;

#[macro_use]
mod vec3;
mod bounding_box;

use crate::object::{Object, Sphere};
use crate::ray::Ray;
use crate::vec3::Vec3;
use rand::{
    self,
    distributions::{Distribution, Uniform},
    Rng,
};

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn of_rgb(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b }
    }

    pub fn black() -> Self {
        Self::of_rgb(0., 0., 0.)
    }

    pub fn white() -> Self {
        Self::of_rgb(1., 1., 1.)
    }

    pub fn add(&self, c: &Color) -> Color {
        Self::of_rgb(self.r + c.r, self.g + c.g, self.b + c.b)
    }

    pub fn mult(&self, c: f64) -> Color {
        Self::of_rgb(self.r * c, self.g * c, self.b * c)
    }

    pub fn mult_(&self, c: &Color) -> Color {
        Self::of_rgb(self.r * c.r, self.g * c.g, self.b * c.b)
    }

    const MAX_VAL: i32 = 255;

    pub fn to_int_rgb(&self) -> (i32, i32, i32) {
        let norm = |c: f64| (c * Self::MAX_VAL as f64).round() as i32;

        (norm(self.r), norm(self.g), norm(self.b))
    }
}

struct Image {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<Color>>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![vec![Color::black(); width]; height];

        Self {
            width,
            height,
            buffer,
        }
    }

    pub fn to_ppm(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        write!(
            file,
            "P3\n{} {}\n{}\n",
            self.width,
            self.height,
            Color::MAX_VAL
        )?;

        for row in self.buffer.iter().rev() {
            for col in row {
                let (r, g, b) = col.to_int_rgb();
                write!(file, "{} {} {} ", r, g, b)?;
            }
            write!(file, "\n")?;
        }

        Ok(())
    }

    pub fn color(&mut self, x: usize, y: usize, col: Color) {
        self.buffer[y][x] = col;
    }
}

#[derive(Debug, Clone, Copy)]
struct Camera {
    origin: Point3,
    look_dir: Vec3,
    up_dir: Vec3,
    viewport_width: usize,
    viewport_height: usize,
    fov: f64,
    focus_dist: f64,
}

impl Camera {
    pub fn new(
        origin: Point3,
        look_point: Point3,
        up: Vec3,
        viewport_width: usize,
        viewport_height: usize,
        fov: f64,
    ) -> Self {
        let look_dir = (look_point - origin).unit_vec();
        let up_dir = up.proj_onto_plane(&look_dir).unit_vec();

        Camera {
            origin,
            look_dir,
            up_dir,
            viewport_width,
            viewport_height,
            fov: fov.to_radians(),
            focus_dist: (look_point - origin).mag(),
        }
    }

    pub fn cast_ray(&self, x: i32, y: i32) -> Ray {
        // distance in front of the camera of the projection plane
        //
        let lense_radius = 0.2;

        let focus_point = self.origin + self.look_dir * self.focus_dist;

        let aspect_ratio = self.viewport_width as f64 / self.viewport_height as f64;
        let plane_width = (self.fov / 2.).tan() * self.focus_dist * 2.;
        let plane_height = plane_width * (1. / aspect_ratio);

        let mut rng = rand::thread_rng();
        let x = x as f64 + Uniform::new_inclusive(0., 1. as f64).sample(&mut rng);
        let y = y as f64 + Uniform::new_inclusive(0., 1. as f64).sample(&mut rng);

        let norm_x = ((x / self.viewport_width as f64) - 0.5) * plane_width;
        let norm_y = ((y / self.viewport_height as f64) - 0.5) * plane_height;

        let cross_plane = self.look_dir.cross(&self.up_dir);

        let perp_component = norm_x * cross_plane;
        let up_component = norm_y * self.up_dir;

        let rand_in_circle = Vec3::rand_in_unit_disc();
        let cast_point = perp_component + up_component + focus_point;

        let ray_origin = self.origin
            + lense_radius * rand_in_circle.x() * cross_plane
            + lense_radius * rand_in_circle.y() * self.up_dir;
        let cast_dir = cast_point - ray_origin;

        Ray::new(ray_origin, cast_dir)
    }
}

pub struct Scene {
    cam: Camera,
    objects: Vec<Box<Object>>,
}

impl Scene {
    pub fn color_of_ray(&self, ray: &Ray, max_depth: i32, infinity_color: Color) -> Color {
        if max_depth <= 0 {
            return Color::black();
        }
        let infinity_hit = RayHit {
            col: infinity_color,
            t: f64::INFINITY,
            point: Vec3::empty(),
            normal: Vec3::empty(),
            front_face: true,
            mat: Arc::new(Lambert {
                albedo: Color::of_rgb(0.5, 0.5, 0.5),
            }),
        };

        let closest_hit = self.objects.iter().fold(infinity_hit, |acc, obj| {
            if let Some(hit) = obj.hit(&ray) {
                if hit.t < acc.t {
                    return hit;
                }
            }
            acc
        });

        if closest_hit.t == f64::INFINITY {
            return closest_hit.col;
        }


        let mat = closest_hit.mat.clone();
        let emitted = mat.emit(ray, &closest_hit);

        if let Some((attenuation, bounce)) = mat.scatter(ray, &closest_hit) {
            self.color_of_ray(&bounce, max_depth - 1, infinity_color)
                .mult_(&attenuation)
        } else {
            emitted
        }
    }
}


pub fn plane_scene() -> Vec<Box<dyn Object>> {
    let mut objects: Vec<Box<dyn Object>> = Vec::new();
    objects.push(Box::new(Sphere::new(
        vec3!(-15., 20., 4.),
        4.,
        Color::of_rgb(1., 0., 1.),
        Arc::new(
            DiffuseLight {
            col: Color::of_rgb(0.3, 0.9, 0.3),
            }

        ),
    )));

    objects.push(Box::new(Sphere::new(
        vec3!(-43., 30., 4.),
        4.,
        Color::of_rgb(1., 0., 1.),
        Arc::new(

            DiffuseLight {
            col: Color::of_rgb(0.3, 0.3, 0.9),
            }

        ),
    )));

    objects.push(Box::new(Sphere::new(
        vec3!(-10., 35., 4.),
        4.,
        Color::of_rgb(1., 0., 1.),
        Arc::new(Metal {
            albedo: Color::of_rgb(0.1, 0.1, 0.9),
            fuzz: 0.05,
        }),
    )));

    objects.push(Box::new(Sphere::new(
        vec3!(0., 15., 4.),
        4.,
        Color::of_rgb(0., 1., 0.),
        Arc::new(Glass {
            refraction_index: 1.5,
        }),
    )));

    let mirror = Box::new(Sphere::new(
        vec3!(10., 10., 4.),
        4.,
        Color::of_rgb(0., 1., 1.),
        Arc::new(Metal {
            albedo: Color::of_rgb(0.8, 0.8, 0.8),
            fuzz: 0.,
        }),
    ));
    let focus_point = vec3!(5., 10., 4.);
    objects.push(mirror);

    objects.push(Box::new(Sphere::new(
        vec3!(0., 0., -100000.),
        100000.,
        Color::of_rgb(0.5, 0.5, 0.5),
        Arc::new(
            Lambert {
            albedo: Color::of_rgb(0.5, 0.5, 0.5),
        }),
    )));

    for i in 0..300 {
        let mut rng = rand::thread_rng();

        let rand_material: Arc<dyn Material> = {
            let s = rng.gen_range(0..8);

            let r = rng.gen_range(0.5..1.);
            let g = rng.gen_range(0.5..1.);
            let b = rng.gen_range(0.5..1.);

            match s {
                0 | 1 => Arc::new(Metal {
                    albedo: Color::of_rgb(r, g, b),
                    fuzz: rng.gen_range(0.0..1.),
                }),
                2 | 3 => Arc::new(Lambert {
                    albedo: Color::of_rgb(r, g, b),
                }),
                4 => Arc::new(Glass {
                    refraction_index: 1.5,
                }),
                5 | 6 => Arc::new(DiffuseLight {
                    col: Color::of_rgb(r,g,b),
                }),
                _ => Arc::new(Metal {
                    albedo: Color::of_rgb(1.0, 1.0, 1.0),
                    fuzz: 0.,
                }),
            }
        };

        let x = 3. * rng.gen_range(-20..20) as f64;
        let y = 3. * rng.gen_range(-20..40) as f64;
        let r = 1.; // rng.gen_range(0.0..1.);

        objects.push(Box::new(Sphere::new(
            vec3!(x, y, r),
            r,
            Color::of_rgb(0., 1., 0.),
            rand_material,
        )));
    }

    objects
}
const VIEWPORT_WIDTH: usize = 852;//1280;
const VIEWPORT_HEIGHT: usize = 480; //720;

pub fn cornell_box() -> Arc<Scene> {
    let mut objects: Vec<Box<dyn Object>> = Vec::new();
    let mut focus_point = vec3!(0., 2., 1.);
    let cam = Camera::new(
        vec3!(0., -12., 3.),
        focus_point,
        vec3!(0., 0., 2.),
        VIEWPORT_WIDTH,
        VIEWPORT_HEIGHT,
        50.,
    );

    /*
    objects.push(Box::new(Rect {
        p0: (0., 100.),
        p1: (0., 100.),
        k: 5.,
        axis: Axis::XZ,
        mat: Arc::new(Lambert {
            albedo: Color::of_rgb(1., 0.4, 0.4),
        }),
    }));*/

    // floor
    objects.push(Box::new(Rect {
        p0: (-5., 5.),
        p1: (-5., 5.),
        k: 0.,
        axis: Axis::XY,
        mat: Arc::new(Lambert {
            albedo: Color::of_rgb(0.4, 0.4, 0.4),
        }),
    }));


    objects.push(Box::new(Rect {
        p0: (-5., 5.),
        p1: (-5., 5.),
        k: 4.,
        axis: Axis::XY,
        mat: Arc::new(Lambert {
            albedo: Color::of_rgb(0.4, 0.4, 0.4),
        }),
    }));

    objects.push(Box::new(FlipFace {
        obj:
        Arc::new(Rect {
        p0: (-1.5, 1.5),
        p1: (0., 2.),
        k: 3.9,
        axis: Axis::XY,
        mat: Arc::new(DiffuseLight {
            col: Color::of_rgb(4., 4., 4.),
        }),
        })
    }));

    // back wall
    objects.push(Box::new(Rect {
        p0: (-100., 100.),
        p1: (-100., 100.),
        k: 5.,
        axis: Axis::XZ,
        mat: Arc::new(Lambert {
            albedo: Color::of_rgb(1., 0., 0.),
        }),
    }));

    objects.push(Box::new(Rect {
        p0: (-100., 100.),
        p1: (-100., 100.),
        k: -14.,
        axis: Axis::XZ,
        mat: Arc::new(Lambert {
            albedo: Color::black(),
        }),
    }));


    objects.push(Box::new(Rect {
        p0: (-100., 100.),
        p1: (-100., 100.),
        k: -3.,
        axis: Axis::YZ,
        mat: Arc::new(Lambert {
            albedo: Color::of_rgb(0., 1., 0.),
        }),
    }));

    objects.push(Box::new(Rect {
        p0: (-100., 100.),
        p1: (-100., 100.),
        k: 3.,
        axis: Axis::YZ,
        mat: Arc::new(Lambert {
            albedo: Color::of_rgb(0., 0., 1.),
        }),
    }));

    objects.push(Box::new(Sphere::new(
        vec3!(-1.5, 1., 1.),
        1.,
        Color::of_rgb(0.5, 0.5, 0.5),
        Arc::new(Metal {
            albedo: Color::of_rgb(0.9, 0.9, 0.9),
            fuzz: 0.,
        }),
    )));

    objects.push(Box::new(Sphere::new(
        vec3!(1.5, 1., 1.),
        1.,
        Color::of_rgb(0.5, 0.5, 0.5),
        Arc::new(Glass {
                    refraction_index: 1.52,
                }),
    )));

    objects.push(Box::new(Sphere::new(
        vec3!(0., -1., 0.5),
        0.5,
        Color::of_rgb(0.5, 0.5, 0.5),
        Arc::new(Metal {
            albedo: Color::of_rgb(0.9, 0.0, 0.9),
            fuzz: 1.,
        }),
    )));


    let objects : Vec<Box<dyn Object>> = vec![Box::new(ObjectGroup::create_hierarchy(objects))];
    let scene = Arc::from(Scene { cam, objects });
    scene
}

fn main() {

    let mut img = Image::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let scene = cornell_box();

    let SAMPLES: i32 = 100;

    let lines_complete = AtomicI64::new(0);

    let color_for_pixels = (0..VIEWPORT_HEIGHT)
        .into_par_iter()
        .flat_map(|y| {
            let line = (0..VIEWPORT_WIDTH)
                .map(|x| {
                    let mut color = Color::black();
                    for _ in 0..SAMPLES {
                        let b: f64 = ((y as f64 / VIEWPORT_HEIGHT as f64) + 0.4).min(1.);

                        let ray = scene.cam.cast_ray(x as i32, y as i32);
                        let sky = Color::of_rgb(0.4, 0.4, b);
                        color =
                            color.add(&scene.color_of_ray(&ray, 4, sky));
                    }

                    color = color.mult(1. / SAMPLES as f64);
                    color = Color::of_rgb(color.r.sqrt(), color.g.sqrt(), color.b.sqrt());

                    (x, y, color)
                })
                .collect::<Vec<_>>();
            let lines_complete = lines_complete.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if lines_complete % 50 == 0 {
                let progress = (lines_complete as f64 / VIEWPORT_HEIGHT as f64) * 100.;
                eprintln!("{:?}%", progress);
            }
            line
        })
        .collect::<Vec<_>>();

    for (x, y, col) in color_for_pixels {
        img.color(x, y, col);
    }

    img.to_ppm("test.ppm");
}
