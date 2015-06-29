extern crate clap;
extern crate image;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;

use std::fs::{File};
use std::path::{Path};

use self::na::{Iso3, Pnt3, Vec3, Translate};
use ncollide::shape::{Ball};
use ncollide::ray::{LocalRayCast, Ray, RayCast, RayIntersection};
use ncollide::math::{Point, Vect, Scalar};
use ncollide::bounding_volume::{BoundingSphere, HasBoundingSphere};
use ncollide::partitioning::BVT;

mod camera;
mod light;
mod material;
mod scene;
mod spectrum;
mod surface;

use camera::{Camera, PerspectiveCamera};
use clap::{Arg, App};
use light::{DirectionalLight, Light, PointLight};
use material::{Material};
use scene::{Scene};
use surface::{Diffuse, SurfaceIntegrator};

struct Object {
    mesh: Box<RayCast<Pnt3<f64>, Iso3<f64>>>,
    transform: Iso3<f64>,
    material: Material
}

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

fn render(width: u32, 
          height: u32, 
          camera: &PerspectiveCamera,
          scene: &mut Scene,
          depth: isize) -> Vec<u8> {
    let mut colours = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_from(x, y);
            let c = scene.trace(&ray, depth);
            // constrain rgb components to range [0, 255]
            colours.push(na::clamp(c.x * 255.0, 0.0, 255.0) as u8);
            colours.push(na::clamp(c.y * 255.0, 0.0, 255.0) as u8);
            colours.push(na::clamp(c.z * 255.0, 0.0, 255.0) as u8);
        }
    }
    colours
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

    let mut scene = Scene::new();

    let transform = Iso3::new(Vec3::new(1.0, 0.0, 10.0), na::zero());
    let sphere = Box::new(Sphere::new(Ball::new(1.0), na::one()));
    let bounding_sphere = Box::new(get_bounding_sphere(sphere.ball(), &transform));
    scene.add_sphere(bounding_sphere as Box<LocalRayCast<Pnt3<f64>>>);

    let transform = Iso3::new(Vec3::new(-4.0, 5.0, 15.0), na::zero());
    let sphere = Box::new(Sphere::new(Ball::new(2.0), na::one()));
    let bounding_sphere = Box::new(get_bounding_sphere(sphere.ball(), &transform));
    scene.add_sphere(bounding_sphere as Box<LocalRayCast<Pnt3<f64>>>);

    // let spheres = vec!(
    //     (sphere, bounding_sphere)
    // );

    // let mut world = BVT::new_balanced(spheres);

    let dir_light = Box::new(DirectionalLight::new(0.2, na::one(), Vec3::z()));
    scene.add_light(dir_light as Box<Light>);
    let pnt_light_red = Box::new(PointLight::new(1.0, Vec3::new(1.0, 0.0, 0.0), Pnt3::new(10.0, 0.0, 0.0), 100.0));
    scene.add_light(pnt_light_red as Box<Light>);
    let pnt_light_green = Box::new(PointLight::new(1.0, Vec3::new(0.0, 1.0, 0.0), Pnt3::new(0.0, 5.0, 0.0), 50.0));
    scene.add_light(pnt_light_green as Box<Light>);
    let pnt_light_blue = Box::new(PointLight::new(1.0, Vec3::new(0.0, 0.0, 1.0), Pnt3::new(0.0, 0.0, 10.0), 40.0));
    scene.add_light(pnt_light_blue as Box<Light>);

    let depth = 2;
    let colours = render(width, height, &camera, &mut scene, depth);

    let filename = matches.value_of("OUTPUT").unwrap_or("pbrt.png");
    let ref mut out = File::create(&Path::new(filename)).ok().expect("Could not create image file");
    let img = image::ImageBuffer::from_raw(width, height, colours).expect("Could not create image buffer");
    let _ = image::ImageRgb8(img).save(out, image::PNG);
}