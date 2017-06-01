extern crate nalgebra as na;

use alga::general::Inverse;
use self::na::{Isometry3, Orthographic3, Perspective3, Point3, Translation, Vector4};

use math::{Point, Scalar, Vector};
use ray::Ray;

pub trait Camera {
    fn ray_from(&self, x: Scalar, y: Scalar) -> Ray;
    fn look_at_z(&mut self, at: &Point, up: &Vector);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub struct PerspectiveCamera {
    width: u32,
    height: u32,
    transform: Isometry3<Scalar>,
    proj: Perspective3<Scalar>,
}

impl PerspectiveCamera {
    pub fn new(transform: Isometry3<Scalar>,
               width: u32,
               height: u32,
               fov: Scalar,
               znear: Scalar,
               zfar: Scalar)
               -> PerspectiveCamera {
        PerspectiveCamera {
            width: width,
            height: height,
            transform: transform,
            proj: Perspective3::new((width as Scalar) / (height as Scalar), fov, znear, zfar),
        }
    }
}

impl Camera for PerspectiveCamera {
    fn ray_from(&self, x: Scalar, y: Scalar) -> Ray {
        let viewproj = self.transform.to_homogeneous() * self.proj.as_matrix().inverse();
        let device_x = ((x / self.width as Scalar) - 0.5) * 2.0;
        let device_y = -((y / self.height as Scalar) - 0.5) * 2.0;
        let point = Vector4::new(device_x, device_y, -1.0, 1.0);
        let h_eye = viewproj * point;
        let eye: Point = Point::from_homogeneous(h_eye)
            .expect("Could not convert from homogeneous Vector.");
        let origin = Point3::from_coordinates(self.transform.translation.vector);
        let direction = na::normalize(&(eye - origin));
        Ray::new(origin, direction)
    }

    #[inline]
    fn look_at_z(&mut self, at: &Point, up: &Vector) {
        let origin = Point3::from_coordinates(self.transform.translation.vector);
        // FIXME: this may need to be look_at_rh instead.
        self.transform = Isometry3::look_at_lh(&origin, at, up);
    }

    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }
}

// pub struct OrthographicCamera {
//     width: u32,
//     height: u32,
//     iso: Isometry3<Scalar>,
//     proj: Orthographic3<Scalar>
// }
