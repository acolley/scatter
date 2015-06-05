extern crate clap;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;

use self::na::{Vec3, Mat3, Mat4};
use ncollide::shape::{Ball};
use transform::Transform;

mod camera;
mod transform;

// use clap::{Arg, App, SubCommand};

struct Sphere {
    transform: Transform,
    ball: Ball<f64>
}

impl Sphere {
    fn new(transform: Transform, ball: Ball<f64>) -> Sphere {
        Sphere {
            transform : transform,
            ball : ball
        }
    }
}

fn main() {
    let mut spheres = Vec::new();
    spheres.push(Sphere::new(Transform::identity(), Ball::new(1.0)));
    // let matches = App::new("pbrt")
    //                    .version("1.0")
    //                    .author("acolley <alnessy@hotmail.com>")
    //                    .about("Physically Based Ray Tracer")

    // let vec: Vec3<f64> = nalgebra::zero();
    let mat: Mat3<f64> = na::one();
    // let transformed = mat * vec;
    println!("{:?}", mat);
}