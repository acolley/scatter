extern crate alga;
#[macro_use]
extern crate approx;
#[macro_use]
extern crate bitflags;
extern crate clap;
extern crate image;
extern crate rand;
extern crate serde_json;
extern crate uuid;
extern crate nalgebra as na;
extern crate ncollide;
extern crate tobj;

use std::collections::HashMap;
use std::io::Read;
use std::f64::consts;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use na::{Isometry3, Point2, Point3, Vector3};
use ncollide::shape::{Ball, Cuboid, TriMesh3};

mod assets;
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
use parse::View;
use rand::StdRng;
use renderer::{Renderer, StandardRenderer};
use scene::{Scene, SceneNode};
use spectrum::Spectrum;
use texture::{ConstantTexture, ImageTexture, Texture};

fn load_obj(filename: &Path) -> Vec<TriMesh3<Scalar>> {
    let obj = tobj::load_obj(filename);
    let (models, _materials) = obj.expect("Could not load .obj");
    let mut meshes = Vec::new();

    for model in models {
        let mesh = &model.mesh;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut uvs = Vec::new();

        for i in 0..mesh.indices.len() / 3 {
            indices.push(Point3::new(mesh.indices[i * 3] as usize,
                                     mesh.indices[i * 3 + 1] as usize,
                                     mesh.indices[i * 3 + 2] as usize));
        }

        for v in 0..mesh.positions.len() / 3 {
            vertices.push(Point3::new(mesh.positions[v * 3] as Scalar,
                                      mesh.positions[v * 3 + 1] as Scalar,
                                      mesh.positions[v * 3 + 2] as Scalar));
        }

        for t in 0..mesh.texcoords.len() / 2 {
            uvs.push(Point2::new(mesh.texcoords[t * 2] as Scalar,
                                 mesh.texcoords[t * 2 + 1] as Scalar));
        }

        let normals = if mesh.normals.is_empty() {
            let mut normals = Vec::new();
            for idx in &indices {
                let v1 = vertices[idx.x];
                let v2 = vertices[idx.y];
                let v3 = vertices[idx.z];
                normals.push((v2 - v1).cross(&(v3 - v1)));
            }
            Some(Arc::new(normals))
        } else {
            let mut normals = Vec::new();
            for n in 0..mesh.normals.len() / 3 {
                normals.push(Vector3::new(mesh.normals[n * 3] as Scalar,
                                          mesh.normals[n * 3 + 1] as Scalar,
                                          mesh.normals[n * 3 + 2] as Scalar));
            }
            Some(Arc::new(normals))
        };

        let uvs = if uvs.is_empty() {
            None
        } else {
            Some(Arc::new(uvs))
        };

        meshes.push(TriMesh3::new(Arc::new(vertices), Arc::new(indices), uvs, normals))
    }
    meshes
}

fn render(width: u32,
          height: u32,
          nthreads: u32,
          samples_per_pixel: u32,
          camera: &Arc<Camera + Sync + Send>,
          scene: &Arc<Scene>,
          renderer: &Arc<Renderer + Sync + Send>)
          -> Vec<u8> {
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
            let mut rng = StdRng::new().expect("Could not create random number generator");
            // let rng = StdRng.from_seed();
            for x in xstart..xend {
                for y in 0..height {
                    let mut c = if samples_per_pixel == 1 {
                        let ray = camera.ray_from(x as Scalar, y as Scalar);
                        renderer.render(&ray, &scene, &mut rng)
                    } else {
                        (0..samples_per_pixel).map(|_| {
                            // TODO: make the sampling methods into their
                            // own trait/struct implementations for different
                            // types of samplers to be used interchangeably
                            let dx = rand::random::<Scalar>() - 0.5;
                            let dy = rand::random::<Scalar>() - 0.5;
                            let ray = camera.ray_from((x as Scalar) + dx, (y as Scalar) + dy);
                            renderer.render(&ray, &scene, &mut rng)
                        }).fold(na::zero(), |sum, c| sum + c)
                    };
                    c = c / (samples_per_pixel as Scalar);
                    tx.send((x, y, c))
                        .expect(&format!("Could not send Spectrum value for ({}, {})", x, y));
                }
            }
        });
    }
    let mut pixel_map: HashMap<(u32, u32), Spectrum> = HashMap::with_capacity((width * height) as
                                                                              usize);

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

fn setup_scene<P: AsRef<Path>>(filename: P) -> (Scene, HashMap<String, View>) {
    let mut f = File::open(filename).expect("Could not open scene file.");
    let mut json_str = String::new();
    f.read_to_string(&mut json_str);

    match parse::parse_scene(&json_str) {
        Ok(res) => res,
        Err(err) => panic!("{}", err),
    }
}

fn main() {
    let matches = App::new("pbrt")
        .version("0.1")
        .arg(Arg::with_name("SCENE").required(true))
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

    let width = matches.value_of("WIDTH")
        .unwrap_or("100")
        .parse::<u32>()
        .expect("Value for width is not a valid unsigned integer");
    let height = matches.value_of("HEIGHT")
        .unwrap_or("100")
        .parse::<u32>()
        .expect("Value for height is not a valid unsigned integer");
    let samples = matches.value_of("SAMPLES")
        .unwrap_or("3")
        .parse::<u32>()
        .expect("Value for samples is not a valid unsigned integer");
    assert!(samples > 0);
    let depth = matches.value_of("DEPTH")
        .unwrap_or("6")
        .parse::<i32>()
        .expect("Value for depth is not a valid signed integer");
    let nthreads = matches.value_of("THREADS")
        .unwrap_or("1")
        .parse::<u32>()
        .expect("Value for threads is not a valid unsigned integer");
    assert!(nthreads > 0);

    let scene_filename = matches.value_of("SCENE").unwrap();

    let (scene, views) = setup_scene(&scene_filename);
    let scene = Arc::new(scene);

    for (name, view) in &views {
        let colours = render(view.camera.width(),
                             view.camera.height(),
                             nthreads,
                             view.samples,
                             &view.camera,
                             &scene,
                             &view.renderer);
        let filename = matches.value_of("OUTPUT").unwrap_or(name);
        let out =
            &mut File::create(&Path::new(filename)).expect("Could not create image file");
        let img = image::ImageBuffer::from_raw(width, height, colours)
            .expect("Could not create image buffer");
        let _ = image::ImageRgb8(img).save(out, image::PNG);
    }
}
