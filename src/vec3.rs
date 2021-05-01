use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    a: [f64; 3],
}

pub type Point3 = Vec3;

impl Vec3 {
    pub fn empty() -> Self {
        Self { a: [0., 0., 0.] }
    }

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { a: [x, y, z] }
    }

    pub fn x(&self) -> f64 {
        self.a[0]
    }

    pub fn y(&self) -> f64 {
        self.a[1]
    }

    pub fn z(&self) -> f64 {
        self.a[2]
    }

    pub fn of_scalar(x: f64) -> Self {
        Self::new(x, x, x)
    }

    pub fn dot(self, other: &Self) -> f64 {
        self.a.iter().zip(other.a.iter()).map(|(a, b)| a * b).sum()
    }

    pub fn cross(&self, other: &Self) -> Self {
        let a = self.a;
        let b = other.a;
        Self {
            a: [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ],
        }
    }

    pub fn mag_squared(&self) -> f64 {
        self.a.iter().map(|a| a * a).sum()
    }

    pub fn mag(&self) -> f64 {
        self.mag_squared().sqrt()
    }

    pub fn dot_(a: &Vec3, b: &Vec3) -> f64 {
        a.dot(b)
    }

    pub fn cross_(a: &Vec3, b: &Vec3) -> Self {
        a.cross(b)
    }

    pub fn unit_vec(self) -> Self {
        self / self.mag()
    }
}

#[macro_export]
macro_rules! vec3 {
    ($a:expr,$b:expr,$c:expr) => {
        Vec3::new($a, $b, $c)
    };
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            a: [
                self.a[0] + other.a[0],
                self.a[1] + other.a[1],
                self.a[2] + other.a[2],
            ],
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            a: [-self.a[0], -self.a[1], -self.a[2]],
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, fact: f64) -> Self {
        Self {
            a: [self.a[0] * fact, self.a[1] * fact, self.a[2] * fact],
        }
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, fact: f64) -> Self {
        self * (1. / fact)
    }
}

#[test]
fn test_ops() {
    let a = Vec3::new(3., 0., 2.);
    let b = Vec3::new(-1., 4., 2.);

    assert_eq!(a.cross(&b), Vec3::new(-8., -8., 12.));
    assert_eq!(a.dot(&b), 1.);
    assert_eq!(a + b, Vec3::new(2., 4., 4.));
    assert_eq!(-a, Vec3::new(-3., -0., -2.));
    assert_eq!(-a * 2., Vec3::new(-6., -0., -4.));
    assert_eq!(a / 2., Vec3::new(1.5, -0., 1.));
    assert_eq!(a - b, Vec3::new(4., -4., 0.));
    assert_eq!(a.mag_squared(), 13.);
    assert_eq!(a.mag(), (13 as f64).sqrt());
}
