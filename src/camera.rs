extern crate nalgebra as na;

use self::na::{Iso3, OrthoMat3, PerspMat3, Pnt4, Translation};

use math::{Point, Scalar, Vector};
use ray::{Ray};

pub trait Camera {
    fn ray_from(&self, x: Scalar, y: Scalar) -> Ray;
    fn look_at_z(&mut self, at: &Point, up: &Vector);
}

pub struct PerspectiveCamera {
    width: u32,
    height: u32,
    transform: Iso3<Scalar>,
    proj: PerspMat3<Scalar>
}

impl PerspectiveCamera {
    pub fn new(transform: Iso3<Scalar>,
               width: u32,
               height: u32,
               fov: Scalar,
               znear: Scalar,
               zfar: Scalar) -> PerspectiveCamera {
        PerspectiveCamera {
            width : width,
            height : height,
            transform : transform,
            proj : PerspMat3::new((width as Scalar) / (height as Scalar), fov, znear, zfar)
        }
    }
}

impl Camera for PerspectiveCamera {
    fn ray_from(&self, x: Scalar, y: Scalar) -> Ray {
        let viewproj = na::to_homogeneous(&self.transform) * na::inv(&self.proj.to_mat()).expect("Projection matrix is not invertible");
        let device_x = ((x / self.width as Scalar) - 0.5) * 2.0;
        let device_y = -((y / self.height as Scalar) - 0.5) * 2.0;
        let point = Pnt4::new(device_x, device_y, -1.0, 1.0);
        let h_eye = viewproj * point;
        let eye: Point = na::from_homogeneous(&h_eye);
        Ray::new(self.transform.translation().to_pnt(), na::normalize(&(eye - self.transform.translation().to_pnt())))
    }

    #[inline]
    fn look_at_z(&mut self, at: &Point, up: &Vector) {
        let mut transform = self.transform;
        transform.look_at_z(&self.transform.translation().to_pnt(), at, up);
    }
}

// pub struct OrthographicCamera {
//     width: u32,
//     height: u32,
//     iso: Iso3<Scalar>,
//     proj: OrthoMat3<Scalar>
// }
