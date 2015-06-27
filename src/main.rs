extern crate clap;
extern crate image;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;

use std::fs::{File};
use std::path::{Path};

use self::na::{Iso3, Pnt3, Vec3, Translate};
use ncollide::shape::{Ball};
use ncollide::ray::{LocalRayCast, RayCast, RayIntersection};
use ncollide::math::{Point, Vect, Scalar};
use ncollide::bounding_volume::{BoundingSphere, HasBoundingSphere};
use ncollide::partitioning::BVT;

mod camera;
mod light;

use camera::{Camera, PerspectiveCamera};
use clap::{Arg, App};
use light::{DirectionalLight, Light, PointLight};

struct Sphere {
    ball: Ball<f64>,
    transform: Iso3<f64>
}

/// This is required because the compiler cannot infer enough
/// type information in order to resolve the method 'bounding_sphere'
/// on types that implement HasBoundingSphere (including Ball).
fn get_bounding_sphere<T: HasBoundingSphere<P, M>, P: Point, M: Translate<P>>(t: &T, m: &M) -> BoundingSphere<P> {
    t.bounding_sphere(m)
}

impl Sphere {
    fn new(ball: Ball<f64>, transform: Iso3<f64>) -> Sphere {
        Sphere {
            ball : ball,
            transform : transform
        }
    }

    fn ball(&self) -> &Ball<f64> {
        &self.ball
    }

    fn transform(&self) -> &Iso3<f64> {
        &self.transform
    }
}

fn main() {
    let matches = App::new("pbrt")
                       .version("0.1")
                       .arg(Arg::with_name("OUTPUT")
                            .short("o")
                            .long("output")
                            .takes_value(true))
                       .arg(Arg::with_name("WIDTH")
                            .short("w")
                            .long("width")
                            .takes_value(true))
                       .arg(Arg::with_name("HEIGHT")
                            .short("h")
                            .long("height")
                            .takes_value(true))
                       .get_matches();

    let width = matches.value_of("WIDTH").unwrap_or("100").parse::<u32>().ok().expect("Value for width is not a valid unsigned integer");
    let height = matches.value_of("HEIGHT").unwrap_or("100").parse::<u32>().ok().expect("Value for height is not a valid unsigned integer");

    let mut camera = PerspectiveCamera::new(width, height, 45.0, 1.0, 100000.0);
    camera.look_at_z(&Pnt3::new(0.0, 0.0, 0.0), &Vec3::y());

    let transform = Iso3::new(Vec3::new(1.0, 0.0, 10.0), na::zero());
    let sphere = Box::new(Sphere::new(Ball::new(1.0), na::one()));
    let bounding_sphere = get_bounding_sphere(sphere.ball(), &transform);
    // let spheres = vec!(
    //     (sphere, bounding_sphere)
    // );

    // let mut world = BVT::new_balanced(spheres);

    let dir_light = DirectionalLight::new(1.0, na::one(), Vec3::z());
    let pnt_light = PointLight::new(1.0, Vec3::new(1.0, 0.0, 0.0), Pnt3::new(10.0, 0.0, 0.0), 20.0);
    let mut colours = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_from(x, y);
            match bounding_sphere.toi_and_normal_with_ray(&ray, true) {
                Some(isect) => {
                    let p = ray.orig + ray.dir * isect.toi;
                    // incorporate colour from the object itself
                    // let c = dir_light.sample(&p, &isect.normal);
                    let c = pnt_light.sample(&p, &isect.normal);
                    colours.push(na::clamp((c.x * 255.0) as u8, 0, 255));
                    colours.push(na::clamp((c.y * 255.0) as u8, 0, 255));
                    colours.push(na::clamp((c.z * 255.0) as u8, 0, 255));
                },
                None => { colours.push(0); colours.push(0); colours.push(0); }
            }
            // world.cast_ray(&ray);
        }
    }

    let filename = matches.value_of("OUTPUT").unwrap_or("pbrt.png");
    let ref mut out = File::create(&Path::new(filename)).ok().expect("Could not create image file");
    let img = image::ImageBuffer::from_raw(width, height, colours).expect("Could not create image buffer");
    let _ = image::ImageRgb8(img).save(out, image::PNG);
}