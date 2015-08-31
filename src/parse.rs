
use std::collections::{BTreeMap, HashMap};
use std::path::{Path};
use std::sync::Arc;

use image;
use na;
use na::{Iso3};
use ncollide::bounding_volume::{AABB3, HasAABB};
use ncollide::ray::{RayCast};
use ncollide::shape::{Ball, Cuboid, TriMesh3};
use serde_json;
use serde_json::{Value};

use camera::{Camera, PerspectiveCamera};
use integrator::{Integrator, PathTraced, Whitted};
use light::{Light, PointLight};
use material::{DiffuseMaterial, GlassMaterial, Material, MirrorMaterial};
use math::{Point, Scalar, Vector};
use renderer::{Renderer, StandardRenderer};
use scene::{Scene, SceneNode};
use spectrum::{Spectrum};
use texture::{ConstantTexture, ImageTexture, Texture};

pub struct View {
    camera: Arc<Camera>,
    samples: u32,
    depth: i32,
    renderer: Box<Renderer>
}

impl View {
    pub fn new(camera: Arc<Camera>,
               samples: u32,
               depth: i32,
               renderer: Box<Renderer>) -> View {
        View {
            camera : camera,
            samples : samples,
            depth : depth,
            renderer : renderer
        }
    }
}

// TODO: use proper error handling here, i.e. Result

/// Parse the scene description from a JSON formatted string.
pub fn parse_scene(json: &str) -> (Scene, HashMap<String, View>) {
	let data: Value = serde_json::from_str(json).ok().expect("String not formatted as JSON");

    let root = data.as_object().expect("Root should be a JSON Object");

    let cameras = root.get("cameras").expect("Expected key 'cameras' not found.");
    let views = root.get("views").expect("Expected key 'views' not found.");
    let objects = root.get("objects").expect("Expected key 'objects' not found.");
    let materials = root.get("materials").expect("Expected key 'materials' not found.");
    let lights = root.get("lights").expect("Expected key 'lights' not found.");

    let cameras = parse_cameras(cameras);
    let views = parse_views(views, &cameras);
    let materials = parse_materials(materials);
    let objects = parse_objects(objects, &materials);
    let lights = parse_lights(lights);

    let mut scene = Scene::new(objects);
    for light in lights {
        scene.add_light(light);
    }
    (scene, views)
}

fn parse_cameras(data: &Value) -> HashMap<String, Arc<Camera>> {
    let data = data.as_object().expect("'cameras' should be a JSON Object.");

    let mut cameras = HashMap::new();
    for (name, value) in data.iter() {
        let camera = parse_camera(value);
        cameras.insert(name.clone(), camera);
    }
    cameras
}

fn parse_camera(data: &Value) -> Arc<Camera> {
    let data = data.as_object().expect("Camera should be a JSON Object.");

    let transform = data.get("transform").expect("Camera requires 'transform' key.");
    let transform = parse_transform(transform);

    let width = data.get("width").expect("Camera requires 'width' key.")
                                 .as_u64().expect("Width should be a number.");
    let height = data.get("height").expect("Camera requires 'height' key.")
                                   .as_u64().expect("Height should be a number.");

    let fov = data.get("fov").expect("Camera requires 'fov' key.")
                             .as_f64().expect("fov should be a number.");

    let near = data.get("near").expect("Camera requires 'near' key.")
                               .as_f64().expect("near should be a number.");
    let far = data.get("far").expect("Camera requires 'far' key.")
                             .as_f64().expect("far should be a number.");

    let camera_type = data.get("type")
                          .expect("Camera requires 'type' key.")
                          .as_string().expect("Camera type should be a JSON string.");
    match camera_type {
        "Perspective" => Arc::new(PerspectiveCamera::new(transform, 
                                                         width as u32, 
                                                         height as u32, 
                                                         fov.to_radians(), 
                                                         near, 
                                                         far)),
        _ => panic!("Unrecognised camera type: {}", camera_type)
    }
}

fn parse_views(data: &Value, cameras: &HashMap<String, Arc<Camera>>) -> HashMap<String, View> {
    let data = data.as_object().expect("'views' should be a JSON Object.");

    let mut views = HashMap::new();
    for (name, value) in data.iter() {
        let view = parse_view(value, cameras);
        views.insert(name.clone(), view);
    }
    views
}

fn parse_view(data: &Value, cameras: &HashMap<String, Arc<Camera>>) -> View {
    let data = data.as_object().expect("View should be a JSON Object.");

    let camera = data.get("camera")
                     .expect("View requires a 'camera' key.")
                     .as_string().expect("Camera should be a JSON string.");
    let camera = cameras.get(camera)
                        .expect(&format!("View's camera not found: {}", camera));

    let samples = data.get("samples")
                      .expect("View requires a 'samples' key.")
                      .as_i64().expect("Samples should be a number.");
    let depth = data.get("depth")
                    .expect("View requires a 'depth' key.")
                    .as_i64().expect("Samples should be a number.");

    let integrator = data.get("integrator")
                         .expect("View requires an 'integrator' key.")
                         .as_string().expect("Integrator should be a JSON string.");
    let integrator = match integrator {
        "Path" => Box::new(PathTraced::new(depth as i32)) as Box<Integrator + Sync + Send>,
        "Whitted" => Box::new(Whitted::new(depth as i32)) as Box<Integrator + Sync + Send>,
        _ => panic!("Unrecognised integrator: {}", integrator)
    };

    let renderer = data.get("renderer")
                       .expect("View requires a 'renderer' key.")
                       .as_string().expect("Renderer should be a JSON string.");
    let renderer = match renderer {
        "Standard" => Box::new(StandardRenderer::new(integrator)) as Box<Renderer>,
        _ => panic!("Unrecognised renderer: {}", renderer)
    };

    View::new(camera.clone(), samples as u32, depth as i32, renderer)
}

fn parse_materials(data: &Value) -> HashMap<String, Arc<Material + Sync + Send>> {
    let data = data.as_object().expect("'materials' should be an Object");
    let mut materials = HashMap::new();
    for (name, value) in data.iter() {
        let material = parse_material(value);
        materials.insert(name.clone(), material);
    }
    materials
}

fn parse_material(data: &Value) -> Arc<Material + Sync + Send> {
    let data = data.as_object().expect("Material should be a JSON object.");
    let material_type = data.get("type")
                            .expect("No material 'type' defined.")
                            .as_string().expect("Material type should be a JSON string.");
    match material_type {
        "Glass" => Arc::new(GlassMaterial) as Arc<Material + Sync + Send>,
        "Mirror" => Arc::new(MirrorMaterial) as Arc<Material + Sync + Send>,
        "Diffuse" => Arc::new(parse_diffuse_material(data)) as Arc<Material + Sync + Send>,
        _ => panic!("Unrecognised material type: {}", material_type)
    }
}

fn parse_diffuse_material(data: &BTreeMap<String, Value>) -> DiffuseMaterial {
    let texture = data.get("texture").expect("Diffuse material requires 'texture' key.");
    let texture = parse_texture(&texture);
    DiffuseMaterial::new(texture)
}

fn parse_texture(data: &Value) -> Box<Texture + Sync + Send> {
    let data = data.as_object().expect("Texture should be a JSON object.");
    let texture_type = data.get("type")
                           .expect("Texture should define a 'type' key.")
                           .as_string().expect("Texture type should be a JSON string.");
    match texture_type {
        "Constant" => Box::new(parse_constant_texture(data)) as Box<Texture + Sync + Send>,
        "Image" => Box::new(parse_image_texture(data)) as Box<Texture + Sync + Send>,
        _ => panic!("Unrecognised texture type: {}", texture_type)
    }
}

fn parse_constant_texture(data: &BTreeMap<String, Value>) -> ConstantTexture {
    let colour = data.get("colour").expect("Constant texture should define a 'colour' key.");
    let colour = parse_spectrum(colour);
    ConstantTexture::new(colour)
}

fn parse_image_texture(data: &BTreeMap<String, Value>) -> ImageTexture {
    let filename = data.get("filename").expect("Image texture should define a 'filename' key.");
    let filename = filename.as_string().expect("Filename should be a JSON string.");
    // TODO: use a centralised location for loading/storing assets
    let image = Arc::new(image::open(&Path::new(filename))
        .ok().expect(&format!("Could not load image from file: {}", filename)).to_rgb());
    ImageTexture::new(image.clone())
}

// fn parse_objects(data: &Value, materials: &HashMap<String, Arc<Material>>) -> HashMap<String, Arc<SceneNode>> {
fn parse_objects(data: &Value, materials: &HashMap<String, Arc<Material + Sync + Send>>) -> Vec<Arc<SceneNode>> {
    let data = data.as_object().expect("Objects should be JSON Object.");

    // let mut objects = HashMap::new();
    let mut objects = Vec::new();
    for (name, value) in data.iter() {
        let object = Arc::new(parse_object(value, materials));
        // objects.insert(*name, object);
        objects.push(object);
    }
    objects
}

fn parse_object(data: &Value, materials: &HashMap<String, Arc<Material + Sync + Send>>) -> SceneNode {
    let data = data.as_object().expect("Object should be a JSON Object.");

    let material = data.get("material")
                       .expect("Object should have a 'material' key.")
                       .as_string().expect("Material should be a JSON string.");
    let material = materials.get(material)
                            .expect(&format!("No Material found with name: {}", material));

    let transform = data.get("transform")
                        .expect("Object should have a 'transform' key.");
    let transform = parse_transform(transform);


    let shape = data.get("shape")
                    .expect("Object should have a 'shape' key.")
                    .as_string().expect("Shape should be a JSON string.");
    let (shape, aabb) = match shape {
        "Cuboid" => parse_cuboid(data, &transform),
        "Shape" => parse_sphere(data, &transform),
        _ => panic!("Unrecognised shape: {}", shape)
    };

    SceneNode::new(transform, material.clone(), shape, aabb)
}

fn parse_cuboid(data: &BTreeMap<String, Value>, transform: &Iso3<Scalar>) -> (Box<RayCast<Point, Iso3<Scalar>> + Sync + Send>, AABB3<Scalar>) {
    let extents = data.get("extents").expect("Cuboid shape should have an 'extents' key.");
    let extents = parse_vector(extents);

    let cuboid = Cuboid::new(extents);
    let aabb = cuboid.aabb(transform);
    (Box::new(cuboid) as Box<RayCast<Point, Iso3<Scalar>> + Sync + Send>, aabb)
}

fn parse_sphere(data: &BTreeMap<String, Value>, transform: &Iso3<Scalar>) -> (Box<RayCast<Point, Iso3<Scalar>> + Sync + Send>, AABB3<Scalar>) {
    let radius = data.get("radius").expect("Sphere shape should have a 'radius' key.");
    let radius = radius.as_f64().expect("Radius should be a number.");

    let ball = Ball::new(radius);
    let aabb = ball.aabb(transform);
    (Box::new(ball) as Box<RayCast<Point, Iso3<Scalar>> + Sync + Send>, aabb)
}

fn parse_lights(data: &Value) -> Vec<Box<Light + Sync + Send>> {
    let data = data.as_object().expect("Lights should be a JSON Object.");

    let mut lights = Vec::new();
    for (_, value) in data.iter() {
        let light = parse_light(value);
        lights.push(light);
    }
    lights
}

fn parse_light(data: &Value) -> Box<Light + Sync + Send> {
    let data = data.as_object().expect("Light should be a JSON Object.");

    let light_type = data.get("type").expect("Light should define a 'type' key.");
    let light_type = light_type.as_string().expect("Light type should be a JSON string.");

    let colour = data.get("colour").expect("Light should define a 'colour' key.");
    let colour = parse_spectrum(colour);

    match light_type {
        "Point" => Box::new(parse_point_light(data, colour)) as Box<Light + Sync + Send>,
        _ => panic!("Unrecognised light type: {}", light_type)
    }
}

fn parse_point_light(data: &BTreeMap<String, Value>, colour: Spectrum) -> PointLight {
    let position = data.get("position")
                       .expect("Point light should define a 'position' key.");
    let position = parse_point(position);

    let radius = data.get("radius")
                     .expect("Point light should define a 'radius' key.")
                     .as_f64().expect("Point light radius should be a number.");

    PointLight::new(1.0, colour, position, radius)
}

fn parse_vector(data: &Value) -> Vector {
    let data = data.as_array().expect("Vector should be a JSON array");
    if data.len() != 3 {
        panic!("Vector should be an array of three elements");
    }

    let x = data[0].as_f64().expect("Vector x component should be a number.");
    let y = data[1].as_f64().expect("Vector y component should be a number.");
    let z = data[2].as_f64().expect("Vector z component should be a number.");

    Vector::new(x, y, z)
}

fn parse_point(data: &Value) -> Point {
    let data = data.as_array().expect("Point should be a JSON array");
    if data.len() != 3 {
        panic!("Point should be an array of three elements");
    }

    let x = data[0].as_f64().expect("Point x component should be a number.");
    let y = data[1].as_f64().expect("Point y component should be a number.");
    let z = data[2].as_f64().expect("Point z component should be a number.");

    Point::new(x, y, z)
}

fn parse_transform(data: &Value) -> Iso3<Scalar> {
    let data = data.as_object().expect("Transform should be a JSON object.");

    let position = data.get("position").expect("Transform should have a key 'position'.");
    let position = parse_vector(position);

    let rotation = match data.get("rotation") {
        Some(_) => parse_vector(data.get("rotation").unwrap()),
        None => na::zero()
    };

    Iso3::new(position, rotation)
}

fn parse_spectrum(data: &Value) -> Spectrum {
    let data = data.as_array().expect("Spectrum should be a JSON array.");
    if data.len() != 3 {
        panic!("Spectrum should be an array of three elements");
    }

    let r = data[0].as_f64().expect("Spectrum r component should be a number.");
    let g = data[1].as_f64().expect("Spectrum g component should be a number.");
    let b = data[2].as_f64().expect("Spectrum b component should be a number.");

    Spectrum::new(r, g, b)
}
