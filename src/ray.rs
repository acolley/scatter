
use na::{Pnt3, Vec3};
use ncollide::ray::{Ray3};

#[derive(Clone)]
pub struct Ray {
    pub ray: Ray3<f64>,
	pub depth: i32
}

impl Ray {
    pub fn new(orig: Pnt3<f64>, dir: Vec3<f64>) -> Ray {
        Self::new_with_depth(orig, dir, 0)
    }

    pub fn new_with_depth(orig: Pnt3<f64>, dir: Vec3<f64>, depth: i32) -> Ray {
        Ray {
            ray : Ray3::new(orig, dir),
            depth : depth
        }
    }

    #[inline]
    pub fn orig<'a>(&'a self) -> &'a Pnt3<f64> {
        &self.ray.orig
    }

    #[inline]
    pub fn dir<'a>(&'a self) -> &'a Vec3<f64> {
        &self.ray.dir
    }
}
