extern crate nalgebra as na;

use alga::general::Inverse;
use na::{Isometry3, Matrix4, Orthographic3, Perspective3, Point3, Translation, Vector4};

use math::{Point, Scalar, Vector};
use ray::Ray;

pub trait Camera {
    fn look_at_z(&mut self, at: &Point, up: &Vector);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn view(&self) -> Matrix4<Scalar>;
    fn proj(&self) -> &Matrix4<Scalar>;

    fn position(&self) -> Point3<Scalar>;

    /// Unproject a point from 2D Screen Space to 3D World Space.
    fn unproject(&self, x: Scalar, y: Scalar) -> Point3<Scalar> {
        let viewproj = self.view() * self.proj().inverse();
        // Device coordinates are normalised [-1, 1].
        let device_x = ((x / self.width() as Scalar) - 0.5) * 2.0;
        let device_y = -((y / self.height() as Scalar) - 0.5) * 2.0;
        let point = Vector4::new(device_x, device_y, -1.0, 1.0);
        let h_eye = viewproj * point;
        Point3::from_homogeneous(h_eye).expect("Could not convert from homogeneous Vector.")
    }

    fn ray_from(&self, x: Scalar, y: Scalar) -> Ray {
        let eye = self.unproject(x, y);
        let origin = self.position();
        let direction = na::normalize(&(eye - origin));
        Ray::new(origin, direction)
    }
}

// TODO: cache view and projection matrices for optimisation?
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
    #[inline]
    fn look_at_z(&mut self, at: &Point, up: &Vector) {
        // FIXME: this may need to be look_at_rh instead.
        self.transform = Isometry3::look_at_lh(&self.position(), at, up);
    }

    #[inline]
    fn position(&self) -> Point3<Scalar> {
        Point3::from_coordinates(self.transform.translation.vector)
    }

    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    fn view(&self) -> Matrix4<Scalar> {
        self.transform.to_homogeneous()
    }

    #[inline]
    fn proj(&self) -> &Matrix4<Scalar> {
        self.proj.as_matrix()
    }
}

// pub struct OrthographicCamera {
//     width: u32,
//     height: u32,
//     iso: Isometry3<Scalar>,
//     proj: Orthographic3<Scalar>
// }
