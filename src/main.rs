extern crate clap;
extern crate image;
extern crate rand;
extern crate uuid;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;

use std::fs::{File};
use std::path::{Path};
use std::sync::Arc;

use self::na::{Iso3, Pnt3, Vec3, Translate};
use ncollide::shape::{Ball, Cuboid, Plane};
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
use scene::{Scene, SceneNode};
use surface::{Diffuse, PerfectSpecular};

/// This is required because the compiler cannot infer enough
/// type information in order to resolve the method 'bounding_sphere'
/// on types that implement HasBoundingSphere (including Ball).
fn get_bounding_sphere<T: HasBoundingSphere<P, M>, P: Point, M: Translate<P>>(t: &T, m: &M) -> BoundingSphere<P> {
    t.bounding_sphere(m)
}

fn render(width: u32, 
          height: u32, 
          camera: &PerspectiveCamera,
          scene: &mut Scene,
          depth: isize) -> Vec<u8> {
    let mut colours = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let ray = camera.ray_from(x as f64, y as f64);
            let c = scene.trace(&ray, depth);

            // TODO: make the sampling methods into their
            // own trait/struct implementations for different
            // types of samplers to be used interchangeably

            // cast ray differentials to soften edges
            // and reduce aliasing using random sampling
            let dx = rand::random::<f64>() - 0.5;
            let dy = rand::random::<f64>() - 0.5;
            let ray1 = camera.ray_from((x as f64) + dx, (y as f64) + dy);
            let c1 = scene.trace(&ray1, depth);
            let dx = rand::random::<f64>() - 0.5;
            let dy = rand::random::<f64>() - 0.5;
            let ray2 = camera.ray_from((x as f64) + dx, (y as f64) + dy);
            let c2 = scene.trace(&ray2, depth);
            let c = c * 0.6 + c1 * 0.2 + c2 * 0.2;

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

    let mut camera = PerspectiveCamera::new(Iso3::new(Vec3::new(0.0, 0.0, 0.0), na::zero()), width, height, 45.0, 1.0, 100000.0);
    camera.look_at_z(&Pnt3::new(0.0, 0.0, 0.0), &Vec3::y());

    let mut scene = Scene::new();

    let transform = Iso3::new(Vec3::new(1.0, 0.0, 10.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform, 
                                           Box::new(Diffuse), 
                                           Box::new(Ball::new(1.0)),
                                           true)));

    let transform = Iso3::new(Vec3::new(-4.0, 3.0, 10.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform, 
                                           Box::new(PerfectSpecular), 
                                           Box::new(Ball::new(2.0)),
                                           true)));

    let transform = Iso3::new(Vec3::new(-1.0, -2.0, 5.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform, 
                                           Box::new(Diffuse), 
                                           Box::new(Ball::new(1.0)),
                                           true)));

    let transform = Iso3::new(Vec3::new(0.0, -100.0, 0.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           Box::new(Diffuse),
                                           Box::new(Ball::new(50.0)),
                                           true)));

    let transform = Iso3::new(Vec3::new(0.0, -3.0, 10.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           Box::new(Diffuse),
                                           Box::new(Cuboid::new(Vec3::new(10.0, 0.1, 10.0))),
                                           false)));

    let dir_light = Box::new(DirectionalLight::new(0.8, na::one(), -Vec3::y()));
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