extern crate nalgebra as na;

use self::na::{PerspMat3};
use transform::Transform;

// pub trait Camera {
//     fn ray(&self) -> PRay;
// }

pub struct PerspectiveCamera {
    transform : Transform,
    projection : PerspMat3<f64>
}

impl PerspectiveCamera {
    pub fn new(aspect: f64, fov: f64, znear: f64, zfar: f64) -> PerspectiveCamera {
        PerspectiveCamera {
            transform: Transform::identity(),
            projection: PerspMat3::new(aspect, fov, znear, zfar)
        }
    }

    // fn ray(&self) -> 
}

// impl Camera for PerspectiveCamera {
//     pub fn ray(&self) -> PRay {

//     }
// }