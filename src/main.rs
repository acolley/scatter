#[macro_use]
extern crate bitflags;
extern crate clap;
extern crate image;
extern crate rand;
extern crate uuid;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;

use std::f64::consts;
use std::fs::{File};
use std::path::{Path};
use std::sync::Arc;

use self::na::{Iso3, Pnt3, Vec3, Translate};
use ncollide::ray::{Ray3};
use ncollide::shape::{Ball, Cuboid};
use ncollide::math::{Point};
use ncollide::bounding_volume::{BoundingSphere, HasBoundingSphere};

mod bxdf;
mod camera;
mod light;
mod material;
mod math;
mod ray;
mod renderer;
mod scene;
mod spectrum;
mod texture;

use camera::{Camera, PerspectiveCamera};
use clap::{Arg, App};
use light::{DirectionalLight, PointLight};
use material::{DiffuseMaterial, GlassMaterial, MirrorMaterial};
use renderer::{Renderer, StandardRenderer};
use scene::{Scene, SceneNode};
use texture::{ConstantTexture, ImageTexture};

fn sample(x: f64,
          y: f64,
          camera: &PerspectiveCamera,
          scene: &mut Scene,
          renderer: &Renderer) -> Vec3<f64> {
    let ray = camera.ray_from(x, y);
    renderer.render(&ray, scene)
}

fn render(width: u32, 
          height: u32, 
          samples_per_pixel: u32,
          camera: &PerspectiveCamera,
          scene: &mut Scene,
          renderer: &Renderer) -> Vec<u8> {
    let mut colours = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let mut c: Vec3<f64> = na::zero();
            if samples_per_pixel == 1 {
                c = sample(x as f64, y as f64, camera, scene, renderer);
            } else {
                for _ in 0..samples_per_pixel {
                    // TODO: make the sampling methods into their
                    // own trait/struct implementations for different
                    // types of samplers to be used interchangeably
                    let dx = rand::random::<f64>() - 0.5;
                    let dy = rand::random::<f64>() - 0.5;

                    c = c + sample((x as f64) + dx, (y as f64) + dy, camera, scene, renderer);
                }
            }
            c = c / (samples_per_pixel as f64);
            // constrain rgb components to range [0, 255]
            colours.push(na::clamp(c.x * 255.0, 0.0, 255.0) as u8);
            colours.push(na::clamp(c.y * 255.0, 0.0, 255.0) as u8);
            colours.push(na::clamp(c.z * 255.0, 0.0, 255.0) as u8);
        }
    }
    colours
}

fn setup(scene: &mut Scene) {
    let teximg = Arc::new(image::open(&Path::new("resources/checker_huge.gif")).unwrap().to_rgb());

    let white = Vec3::new(1.0, 1.0, 1.0);
    let yellow = Vec3::new(1.0, 1.0, 0.5);
    let material_yellow = Arc::new(DiffuseMaterial::new(Box::new(ConstantTexture::new(yellow))));
    let material_glass = Arc::new(GlassMaterial);
    let material_reflect = Arc::new(MirrorMaterial);
    let material_white = Arc::new(DiffuseMaterial::new(Box::new(ConstantTexture::new(white))));
    let material_checker = Arc::new(DiffuseMaterial::new(Box::new(ImageTexture::new(teximg.clone()))));

    let transform = Iso3::new(Vec3::new(1.0, 0.0, 10.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform, 
                                           material_yellow.clone(),
                                           Box::new(Ball::new(1.0)))));

    let transform = Iso3::new(Vec3::new(-4.0, 3.0, 10.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform, 
                                           material_glass.clone(),
                                           Box::new(Ball::new(2.0)))));

    // let transform = Iso3::new(Vec3::new(-0.5, -1.0, 7.0), Vec3::new(0.0, 0.0, consts::PI / 4.0));
    // scene.add_node(Arc::new(SceneNode::new(transform,
    //                                        material_reflect.clone(),
    //                                        Box::new(Cuboid::new(Vec3::new(0.5, 0.5, 0.5))))));

    let transform = Iso3::new(Vec3::new(-1.0, -2.0, 5.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform, 
                                           material_checker.clone(),
                                           Box::new(Ball::new(1.0)))));

    // floor
    let transform = Iso3::new(Vec3::new(0.0, -3.0, 0.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           material_white.clone(),
                                           Box::new(Cuboid::new(Vec3::new(50.0, 0.1, 50.0))))));
    // ceiling
    let transform = Iso3::new(Vec3::new(0.0, 47.0, 0.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           material_white.clone(),
                                           Box::new(Cuboid::new(Vec3::new(50.0, 0.1, 50.0))))));
    // front
    let transform = Iso3::new(Vec3::new(0.0, -3.0, 50.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           material_white.clone(),
                                           Box::new(Cuboid::new(Vec3::new(50.0, 50.0, 0.1))))));
    // back
    let transform = Iso3::new(Vec3::new(0.0, -3.0, -50.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           material_white.clone(),
                                           Box::new(Cuboid::new(Vec3::new(50.0, 50.0, 0.1))))));
    // left
    let transform = Iso3::new(Vec3::new(50.0, -3.0, 0.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           material_white.clone(),
                                           Box::new(Cuboid::new(Vec3::new(0.1, 50.0, 50.0))))));
    // right
    let transform = Iso3::new(Vec3::new(-50.0, -3.0, 0.0), na::zero());
    scene.add_node(Arc::new(SceneNode::new(transform,
                                           material_white.clone(),
                                           Box::new(Cuboid::new(Vec3::new(0.1, 50.0, 50.0))))));


    // let dir_light = Box::new(DirectionalLight::new(1.0, na::one(), -Vec3::y()));
    // scene.add_light(dir_light);
    // let pnt_light_red = Box::new(PointLight::new(1.0, Vec3::new(1.0, 0.0, 0.0), Pnt3::new(10.0, 0.0, 0.0), 500.0));
    // scene.add_light(pnt_light_red);
    // let pnt_light_green = Box::new(PointLight::new(1.0, Vec3::new(0.0, 1.0, 0.0), Pnt3::new(0.0, 5.0, 0.0), 500.0));
    // scene.add_light(pnt_light_green);
    // let pnt_light_blue = Box::new(PointLight::new(1.0, Vec3::new(0.0, 0.0, 1.0), Pnt3::new(0.0, 15.0, 10.0), 500.0));
    // scene.add_light(pnt_light_blue);
    let pnt_light_white = Box::new(PointLight::new(1.0, Vec3::new(1.0, 1.0, 1.0), Pnt3::new(0.0, 25.0, 0.0), 600.0));
    scene.add_light(pnt_light_white);
    let pnt_light_white = Box::new(PointLight::new(1.0, Vec3::new(1.0, 1.0, 1.0), Pnt3::new(10.0, 25.0, 10.0), 500.0));
    scene.add_light(pnt_light_white);
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
                       .arg(Arg::with_name("SAMPLES")
                            .short("s")
                            .long("samples")
                            .takes_value(true))
                       .arg(Arg::with_name("DEPTH")
                            .short("d")
                            .long("depth")
                            .takes_value(true))
                       .get_matches();

    let width = matches.value_of("WIDTH").unwrap_or("100").parse::<u32>().ok().expect("Value for width is not a valid unsigned integer");
    let height = matches.value_of("HEIGHT").unwrap_or("100").parse::<u32>().ok().expect("Value for height is not a valid unsigned integer");
    let samples = matches.value_of("SAMPLES").unwrap_or("3").parse::<u32>().ok().expect("Value for samples is not a valid unsigned integer");
    assert!(samples > 0);
    let depth = matches.value_of("DEPTH").unwrap_or("2").parse::<i32>().ok().expect("Value for depth is not a valid unsigned integer");

    let mut camera = PerspectiveCamera::new(Iso3::new(Vec3::new(0.0, 0.0, 0.0), na::zero()), width, height, 45.0, 1.0, 100000.0);
    camera.look_at_z(&Pnt3::new(0.0, 0.0, 0.0), &Vec3::y());

    let mut scene = Scene::new();
    setup(&mut scene);
    let renderer = StandardRenderer::new(depth);

    let colours = render(width, height, samples, &camera, &mut scene, &renderer);

    let filename = matches.value_of("OUTPUT").unwrap_or("pbrt.png");
    let ref mut out = File::create(&Path::new(filename)).ok().expect("Could not create image file");
    let img = image::ImageBuffer::from_raw(width, height, colours).expect("Could not create image buffer");
    let _ = image::ImageRgb8(img).save(out, image::PNG);
}