#[macro_use]
extern crate bitflags;
extern crate clap;
extern crate image;
extern crate rand;
extern crate serde_json;
extern crate uuid;
#[macro_use(assert_approx_eq)]
extern crate nalgebra as na;
extern crate ncollide;
extern crate tobj;

use std::collections::HashMap;
use std::io::{Read};
use std::f64::consts;
use std::fs::{File};
use std::path::{Path};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use self::na::{Iso3, Pnt2, Pnt3, Vec3};
use ncollide::shape::{Ball, Cuboid, TriMesh3};

mod bxdf;
mod camera;
mod integrator;
mod light;
mod material;
mod math;
mod montecarlo;
mod parse;
mod ray;
mod renderer;
mod scene;
mod spectrum;
mod texture;

use camera::{Camera, PerspectiveCamera};
use clap::{Arg, App};
use integrator::{Integrator, Whitted};
use light::{Light, PointLight};
use material::{DiffuseMaterial, GlassMaterial, MirrorMaterial};
use math::{Point, Scalar, Vector};
use rand::{StdRng};
use renderer::{Renderer, StandardRenderer};
use scene::{Scene, SceneNode};
use spectrum::Spectrum;
use texture::{ConstantTexture, ImageTexture, Texture};

fn load_obj(filename: &Path) -> Vec<TriMesh3<Scalar>> {
    let obj = tobj::load_obj(filename);
    let (models, materials) = obj.ok().expect("Could not load .obj");
    let mut meshes = Vec::new();

    for model in models {
        let mesh = &model.mesh;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut uvs = Vec::new();

        for i in 0..mesh.indices.len() / 3 {
            indices.push(
                Pnt3::new(mesh.indices[i * 3] as usize,
                          mesh.indices[i * 3 + 1] as usize,
                          mesh.indices[i * 3 + 2] as usize)
            );
        }

        for v in 0..mesh.positions.len() / 3 {
            vertices.push(
                Pnt3::new(mesh.positions[v * 3] as Scalar,
                          mesh.positions[v * 3 + 1] as Scalar,
                          mesh.positions[v * 3 + 2] as Scalar)
            );
        }

        for t in 0..mesh.texcoords.len() / 2 {
            uvs.push(
                Pnt2::new(mesh.texcoords[t * 2] as Scalar,
                          mesh.texcoords[t * 2 + 1] as Scalar)
            );
        }

        let normals = if mesh.normals.len() > 0 {
            let mut normals = Vec::new();
            for n in 0..mesh.normals.len() / 3 {
                normals.push(
                    Vec3::new(mesh.normals[n * 3] as Scalar,
                              mesh.normals[n * 3 + 1] as Scalar,
                              mesh.normals[n * 3 + 2] as Scalar)
                );
            }
            Some(Arc::new(normals))
        } else {
            let mut normals = Vec::new();
            for idx in indices.iter() {
                let v1 = vertices[idx.x];
                let v2 = vertices[idx.y];
                let v3 = vertices[idx.z];
                normals.push(na::cross(&(v2 - v1), &(v3 - v1)));
            }
            Some(Arc::new(normals))
        };

        let uvs = if uvs.len() > 0 { Some(Arc::new(uvs)) } else { None };

        meshes.push(TriMesh3::new(
            Arc::new(vertices),
            Arc::new(indices),
            uvs,
            normals
        ))
    }
    meshes
}

fn render(
    width: u32,
    height: u32,
    nthreads: u32,
    samples_per_pixel: u32,
    camera: &Arc<PerspectiveCamera>,
    scene: &Arc<Scene>,
    renderer: &Arc<StandardRenderer>) -> Vec<u8> {
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
                    let mut c: Vector = na::zero();
                    if samples_per_pixel == 1 {
                    let ray = camera.ray_from(x as Scalar, y as Scalar);
                        c = renderer.render(&ray, &scene, &mut rng);
                    } else {
                        for _ in 0..samples_per_pixel {
                            // TODO: make the sampling methods into their
                            // own trait/struct implementations for different
                            // types of samplers to be used interchangeably
                            let dx = rand::random::<Scalar>() - 0.5;
                            let dy = rand::random::<Scalar>() - 0.5;
                            let ray = camera.ray_from((x as Scalar) + dx, (y as Scalar) + dy);
                            c = c + renderer.render(&ray, &scene, &mut rng);
                        }
                    }
                    c = c / (samples_per_pixel as Scalar);
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

fn setup_scene(filename: &str) -> Scene {
    let mut f = File::open(filename).ok().expect("Could not open scene file.");
    let mut json_str = String::new();
    f.read_to_string(&mut json_str);

    let (scene, views) = parse::parse_scene(&json_str);
    scene
}

fn main() {
    let matches = App::new("pbrt")
                       .version("0.1")
                       .arg(Arg::with_name("SCENE")
                            .required(true))
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

    let scene_filename = matches.value_of("SCENE").unwrap();

    let mut camera = PerspectiveCamera::new(Iso3::new(Vector::new(0.0, 0.0, -2.5), na::zero()), width, height, consts::FRAC_PI_2, 0.01, 1000.0);
    camera.look_at_z(&Point::new(0.0, 0.0, 0.0), &Vector::y());
    let camera = Arc::new(camera);

    let scene = Arc::new(setup_scene(&scene_filename));
    let integrator = Box::new(Whitted::new(depth));
    // let integrator = Box::new(PathTraced::new(depth));
    let renderer = Arc::new(StandardRenderer::new(integrator as Box<Integrator + Send + Sync>));

    let colours = render(width, height, nthreads, samples, &camera, &scene, &renderer);

    let filename = matches.value_of("OUTPUT").unwrap_or("scatter.png");
    let ref mut out = File::create(&Path::new(filename)).ok().expect("Could not create image file");
    let img = image::ImageBuffer::from_raw(width, height, colours).expect("Could not create image buffer");
    let _ = image::ImageRgb8(img).save(out, image::PNG);
}
