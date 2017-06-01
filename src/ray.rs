
use na::{Point3, Vector3};
use ncollide::query::{Ray3};

#[derive(Clone)]
pub struct Ray {
    pub ray: Ray3<f64>,
	pub depth: i32
}

impl Ray {
    pub fn new(orig: Point3<f64>, dir: Vector3<f64>) -> Ray {
        Self::new_with_depth(orig, dir, 0)
    }

    pub fn new_with_depth(orig: Point3<f64>, dir: Vector3<f64>, depth: i32) -> Ray {
        Ray {
            ray : Ray3::new(orig, dir),
            depth : depth
        }
    }

    #[inline]
    pub fn orig<'a>(&'a self) -> &'a Point3<f64> {
        &self.ray.origin
    }

    #[inline]
    pub fn dir<'a>(&'a self) -> &'a Vector3<f64> {
        &self.ray.dir
    }
}
