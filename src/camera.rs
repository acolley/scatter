extern crate nalgebra as na;

use self::na::{Iso3, Norm, OrthoMat3, PerspMat3, Pnt3, Pnt4, Translation, Vec3};
use ncollide::math::{Point, Scalar, Vect};
use ncollide::ray::{Ray};

pub trait Camera {
    fn ray_from(&self, x: u32, y: u32) -> Ray<Pnt3<f64>>;
}

pub struct PerspectiveCamera {
    width: u32,
    height: u32,
    iso : Iso3<f64>,
    proj : PerspMat3<f64>
}

impl PerspectiveCamera {
    pub fn new(width: u32, height: u32, fov: f64, znear: f64, zfar: f64) -> PerspectiveCamera {
        PerspectiveCamera {
            width : width,
            height : height,
            iso : na::one(),
            proj : PerspMat3::new((width as f64) / (height as f64), fov, znear, zfar)
        }
    }

    pub fn look_at_z(&mut self, at: &Pnt3<f64>, up: &Vec3<f64>) {
        let mut iso = self.iso;
        iso.look_at_z(&self.iso.translation().to_pnt(), at, up);
    }
}

impl Camera for PerspectiveCamera {
    fn ray_from(&self, x: u32, y: u32) -> Ray<Pnt3<f64>> {
        let viewproj = na::to_homogeneous(&self.iso) * na::inv(&self.proj.to_mat()).expect("Projection matrix is not invertible");
        let device_x = ((x as f64 / self.width as f64) - 0.5) * 2.0;
        let device_y = -((y as f64 / self.height as f64) - 0.5) * 2.0;
        let point = Pnt4::new(device_x, device_y, -1.0, 1.0);
        let h_eye = viewproj * point;
        let eye: Pnt3<f64> = na::from_homogeneous(&h_eye);
        Ray::new(self.iso.translation().to_pnt(), na::normalize(&(eye - self.iso.translation().to_pnt())))
    }
}

pub struct OrthographicCamera {
    width: u32,
    height: u32,
    iso: Iso3<f64>,
    proj: OrthoMat3<f64>
}

