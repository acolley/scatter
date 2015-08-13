#[macro_use]
extern crate bitflags;
extern crate clap;
extern crate image;
extern crate rand;
extern crate rustc_serialize;
extern crate uuid;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;

use std::collections::HashMap;
use std::f64::consts;
use std::fs::{File};
use std::path::{Path};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::u32;

use self::na::{Iso3, Pnt3, Vec3, Translate};
use ncollide::ray::{Ray3};
use ncollide::shape::{Ball, Cuboid};
use ncollide::math::{Point};
use ncollide::bounding_volume::{BoundingSphere};

mod bxdf;
mod camera;
mod integrator;
mod light;
mod material;
mod math;
mod montecarlo;
mod ray;
mod renderer;
mod scene;
mod spectrum;
mod texture;

use camera::{Camera, PerspectiveCamera};
use clap::{Arg, App};
use integrator::{Integrator, PathTraced, Whitted};
use light::{DirectionalLight, PointLight};
use material::{DiffuseMaterial, GlassMaterial, MirrorMaterial};
use rand::{StdRng};
use renderer::{Renderer, StandardRenderer};
use scene::{Scene, SceneNode};
use spectrum::Spectrum;
use texture::{ConstantTexture, ImageTexture};

fn render<I>(width: u32, 
             height: u32, 
             nthreads: u32,
             samples_per_pixel: u32,
             camera: &Arc<PerspectiveCamera>,
             scene: &Arc<Scene>,
             renderer: &Arc<StandardRenderer<I>>) -> Vec<u8>
where I: 'static + Integrator + Sync + Send {
    let (tx, rx) = mpsc::channel();
    // partition along the x dimension
    let xchunk_size = width / nthreads;
    for i in 0..nthreads {
        let xstart = i * xchunk_size;
        let xend = f32::min(width as f32, (xstart + xchunk_size) as f32) as u32;

        let tx = tx.clone();
        let camera = camera.clone();
        let scene = scene.clone();
        let renderer = renderer.clone();
        thread::spawn(move || {
            let mut rng = StdRng::new().ok().expect("Could not create random number generator");
            // let rng = StdRng.from_seed();
            for x in xstart..xend {
                for y in 0..height {
                    let mut c: Vec3<f64> = na::zero();
                    if samples_per_pixel == 1 {
                    let ray = camera.ray_from(x as f64, y as f64);
                        c = renderer.render(&ray, &scene, &mut rng);
                    } else {
                        for _ in 0..samples_per_pixel {
                            // TODO: make the sampling methods into their
                            // own trait/struct implementations for different
                            // types of samplers to be used interchangeably
                            let dx = rand::random::<f64>() - 0.5;
                            let dy = rand::random::<f64>() - 0.5;
                            let ray = camera.ray_from((x as f64) + dx, (y as f64) + dy);
                            c = c + renderer.render(&ray, &scene, &mut rng);
                        }
                    }
                    c = c / (samples_per_pixel as f64);
                    tx.send((x, y, c)).ok().expect(&format!("Could not send Spectrum value for ({}, {})", x, y));
                }
            }
        });
    }
    let mut pixel_map: HashMap<(u32, u32), Spectrum> = HashMap::with_capacity((width * height) as usize);

    // explicitly drop the transmission end
    // otherwise the receiver will block indefinitely
    drop(tx);

    for (x, y, c) in rx {
        pixel_map.insert((x, y), c);
    }

    // reconstruct final image
    let mut colours = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            let c = pixel_map.get(&(x, y)).expect(&format!("No pixel at ({}, {})", x, y));
            // constrain rgb components to range [0, 255]
            colours.push(na::clamp(c.x * 255.0, 0.0, 255.0) as u8);
            colours.push(na::clamp(c.y * 255.0, 0.0, 255.0) as u8);
            colours.push(na::clamp(c.z * 255.0, 0.0, 255.0) as u8);
        }
    }

    colours
}

fn setup_scene() -> Scene {
    let teximg = Arc::new(image::open(&Path::new("resources/checker_huge.gif")).unwrap().to_rgb());

    let white = Vec3::new(1.0, 1.0, 1.0);
    let yellow = Vec3::new(1.0, 1.0, 0.5);
    let red = Vec3::new(1.0, 0.0, 0.0);
    let blue = Vec3::new(0.0, 0.0, 1.0);
    let material_yellow = Arc::new(DiffuseMaterial::new(Box::new(ConstantTexture::new(yellow))));
    let material_glass = Arc::new(GlassMaterial);
    let material_reflect = Arc::new(MirrorMaterial);
    let material_white = Arc::new(DiffuseMaterial::new(Box::new(ConstantTexture::new(white))));
    let material_red = Arc::new(DiffuseMaterial::new(Box::new(ConstantTexture::new(red))));
    let material_blue = Arc::new(DiffuseMaterial::new(Box::new(ConstantTexture::new(blue))));
    let material_checker = Arc::new(DiffuseMaterial::new(Box::new(ImageTexture::new(teximg.clone()))));

    let mut nodes = Vec::new();

    let transform = Iso3::new(Vec3::new(1.0, -1.5, 0.8), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform, 
                                       material_reflect.clone(),
                                       Box::new(Ball::new(0.6)))));

    let transform = Iso3::new(Vec3::new(-1.0, -1.5, 0.2), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform, 
                                       material_glass.clone(),
                                       Box::new(Ball::new(0.6)))));

    // let transform = Iso3::new(Vec3::new(-1.0, -1.25, 0.2), na::zero());
    // nodes.push(Arc::new(SceneNode::new(transform, 
    //                                    material_glass.clone(),
    //                                    Box::new(Cuboid::new(Vec3::new(0.5, 0.5, 0.5))))));

    // floor
    let transform = Iso3::new(Vec3::new(0.0, -3.0, 0.0), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform,
                                       material_checker.clone(),
                                       // material_white.clone(),
                                       Box::new(Cuboid::new(Vec3::new(3.0, 0.01, 3.0))))));
    // ceiling
    let transform = Iso3::new(Vec3::new(0.0, 2.9, 0.0), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform,
                                       material_white.clone(),
                                       Box::new(Cuboid::new(Vec3::new(3.0, 0.01, 3.0))))));
    // front
    let transform = Iso3::new(Vec3::new(0.0, 0.0, 3.0), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform,
                                       material_white.clone(),
                                       Box::new(Cuboid::new(Vec3::new(3.0, 3.0, 0.01))))));
    // back
    let transform = Iso3::new(Vec3::new(0.0, 0.0, -3.0), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform,
                                       material_white.clone(),
                                       Box::new(Cuboid::new(Vec3::new(3.0, 3.0, 0.01))))));
    // left
    let transform = Iso3::new(Vec3::new(3.0, 0.0, 0.0), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform,
                                       material_red.clone(),
                                       Box::new(Cuboid::new(Vec3::new(0.01, 3.0, 3.0))))));
    // right
    let transform = Iso3::new(Vec3::new(-3.0, 0.0, 0.0), na::zero());
    nodes.push(Arc::new(SceneNode::new(transform,
                                       material_blue.clone(),
                                       Box::new(Cuboid::new(Vec3::new(0.01, 3.0, 3.0))))));
    let mut scene = Scene::new(nodes);


    // let dir_light = Box::new(DirectionalLight::new(1.0, na::one(), -Vec3::y()));
    // scene.add_light(dir_light);
    // let pnt_light_red = Box::new(PointLight::new(1.0, Vec3::new(1.0, 0.0, 0.0), Pnt3::new(10.0, 0.0, 0.0), 500.0));
    // scene.add_light(pnt_light_red);
    // let pnt_light_green = Box::new(PointLight::new(1.0, Vec3::new(0.0, 1.0, 0.0), Pnt3::new(-20.0, 5.0, 20.0), 20.0));
    // scene.add_light(pnt_light_green);
    // let pnt_light_blue = Box::new(PointLight::new(1.0, Vec3::new(0.0, 0.0, 1.0), Pnt3::new(0.0, 15.0, 10.0), 500.0));
    // scene.add_light(pnt_light_blue);
    let pnt_light_white = Box::new(PointLight::new(1.0, Vec3::new(1.0, 1.0, 1.0), Pnt3::new(0.0, 2.0, 0.0), 8.0));
    scene.add_light(pnt_light_white);
    // let pnt_light_white = Box::new(PointLight::new(1.0, Vec3::new(1.0, 1.0, 1.0), Pnt3::new(10.0, 25.0, 10.0), 500.0));
    // scene.add_light(pnt_light_white);

    scene
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
                       .arg(Arg::with_name("THREADS")
                            .short("t")
                            .long("threads")
                            .takes_value(true))
                       .get_matches();

    let width = matches.value_of("WIDTH").unwrap_or("100").parse::<u32>().ok().expect("Value for width is not a valid unsigned integer");
    let height = matches.value_of("HEIGHT").unwrap_or("100").parse::<u32>().ok().expect("Value for height is not a valid unsigned integer");
    let samples = matches.value_of("SAMPLES").unwrap_or("3").parse::<u32>().ok().expect("Value for samples is not a valid unsigned integer");
    assert!(samples > 0);
    let depth = matches.value_of("DEPTH").unwrap_or("6").parse::<i32>().ok().expect("Value for depth is not a valid signed integer");
    let nthreads = matches.value_of("THREADS").unwrap_or("1").parse::<u32>().ok().expect("Value for threads is not a valid unsigned integer");
    assert!(nthreads > 0);

    let mut camera = PerspectiveCamera::new(Iso3::new(Vec3::new(0.0, 0.0, -2.5), na::zero()), width, height, consts::FRAC_PI_2, 0.01, 1000.0);
    camera.look_at_z(&Pnt3::new(0.0, 0.0, 0.0), &Vec3::y());
    let camera = Arc::new(camera);

    let scene = Arc::new(setup_scene());
    // let integrator = Whitted::new(depth);
    let integrator = PathTraced::new(depth);
    let renderer = Arc::new(StandardRenderer::new(integrator));

    let colours = render(width, height, nthreads, samples, &camera, &scene, &renderer);

    let filename = matches.value_of("OUTPUT").unwrap_or("scatter.png");
    let ref mut out = File::create(&Path::new(filename)).ok().expect("Could not create image file");
    let img = image::ImageBuffer::from_raw(width, height, colours).expect("Could not create image buffer");
    let _ = image::ImageRgb8(img).save(out, image::PNG);
}