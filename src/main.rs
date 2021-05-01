use anyhow::{anyhow, Result};

use std::fs::File;
use std::io::Write;

mod object;
mod ray;
mod vec3;

use crate::object::{Object, Sphere};
use crate::ray::Ray;
use crate::vec3::Vec3;

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

struct Camera {
    origin: Ray,
}

impl Camera {
    pub fn new(origin: Ray) -> Self {
        Camera { origin }
    }
}

struct Scene {
    cam: Camera,
    objects: Vec<Box<Object>>,
}

fn main() {
    const viewport_width: usize = 600;
    const viewport_height: usize = 500;

    let mut objects = Vec::new();

    objects.push(Sphere::new(
        vec3!(0., 5., 1.),
        1.,
        Color::of_rgb(1., 0., 0.),
    ));

    objects.push(Sphere::new(
        vec3!(0., 0., -150.5),
        50.,
        Color::of_rgb(1., 0., 0.),
    ));

    let mut img = Image::new(viewport_width, viewport_height);
    let cam = Camera::new(Ray::new(Vec3::empty(), vec3!(0., 1., 0.)));

    for y in 0..viewport_height {
        for x in 0..viewport_width {
            let proj_x = ((x as f64 / viewport_width as f64) - 0.5) * 5.;
            let proj_z = ((y as f64 / viewport_height as f64) - 0.5) * 5.;

            let proj_point = vec3!(proj_x, 1., proj_z);

            let dir = proj_point - cam.origin.origin;
            let ray = Ray::new(cam.origin.origin, dir);

            let b: f64 = 1.0 - (y as f64 / viewport_height as f64);
            let mut color = Color::of_rgb(0., 0., b);

            for object in &objects {
                if let Some((col, t)) = object.hit(&ray) {
                    color = col;
                }
            }

            img.color(x, y, color);
        }
    }

    img.to_ppm("test.ppm");
}
