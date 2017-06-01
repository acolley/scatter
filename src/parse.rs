
use std::collections::{HashMap};
use std::error;
use std::fmt;
use std::path::{Path};
use std::result;
use std::sync::Arc;

use image;
use na;
use na::{Isometry3};
use ncollide::bounding_volume::{AABB3};
use ncollide::query::{RayCast};
use ncollide::shape::{Ball, Cuboid, Shape, TriMesh3};
use serde_json;
use serde_json::{Map, Value};

use camera::{Camera, PerspectiveCamera};
use integrator::{Integrator, PathTraced, Whitted};
use light::{Light, PointLight};
use material::{DiffuseMaterial, GlassMaterial, Material, MirrorMaterial};
use math::{Point, Scalar, Vector};
use renderer::{Renderer, StandardRenderer};
use scene::{Scene, SceneNode};
use spectrum::{Spectrum};
use texture::{ConstantTexture, ImageTexture, Texture};

// TODO: rewrite in order to use #[derive(Serialize, Deserialize)]

pub type Intersectable = Box<RayCast<Point, Isometry3<Scalar>> + Sync + Send>;

pub struct View {
    pub camera: Arc<Camera + Sync + Send>,
    pub samples: u32,
    pub depth: i32,
    pub renderer: Arc<Renderer + Sync + Send>
}

impl View {
    pub fn new(camera: Arc<Camera + Sync + Send>,
               samples: u32,
               depth: i32,
               renderer: Arc<Renderer + Sync + Send>) -> View {
        View {
            camera,
            samples,
            depth,
            renderer
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ExpectedArray(&'static str),
    ExpectedU64(&'static str),
    ExpectedF64(&'static str),
    ExpectedI64(&'static str),
    ExpectedObject(&'static str),
    ExpectedString(&'static str),
    Json(::serde_json::error::Error),
    MalformedPoint(&'static str),
    MalformedSpectrum(&'static str),
    MalformedVector(&'static str),
    MissingKey(&'static str),
    MissingReference { typ: &'static str, name: &'static str },
    Texture(::image::ImageError)
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ExpectedArray(err) => err,
            Error::ExpectedU64(err) => err,
            Error::ExpectedF64(err) => err,
            Error::ExpectedI64(err) => err,
            Error::ExpectedObject(err) => err,
            Error::ExpectedString(err) => err,
            Error::Json(ref err) => err.description(),
            Error::MalformedPoint(err) => err,
            Error::MalformedSpectrum(err) => err,
            Error::MalformedVector(err) => err,
            Error::MissingKey(err) => err,
            Error::MissingReference { name, .. } => name,
            Error::Texture(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ExpectedArray(_) => None,
            Error::ExpectedU64(_) => None,
            Error::ExpectedF64(_) => None,
            Error::ExpectedI64(_) => None,
            Error::ExpectedObject(_) => None,
            Error::ExpectedString(_) => None,
            Error::Json(ref err) => Some(err),
            Error::MalformedPoint(_) => None,
            Error::MalformedSpectrum(_) => None,
            Error::MalformedVector(_) => None,
            Error::MissingKey(_) => None,
            Error::MissingReference {..} => None,
            Error::Texture(ref err) => Some(err)
        }
    }
}

impl From<::serde_json::error::Error> for Error {
    fn from(err: ::serde_json::error::Error) -> Error {
        Error::Json(err)
    }
}

impl From<::image::ImageError> for Error {
    fn from(err: ::image::ImageError) -> Error {
        Error::Texture(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ExpectedArray(err) => write!(f, "Expected JSON array: {}", err),
            Error::ExpectedU64(err) => write!(f, "Expected JSON u64: {}", err),
            Error::ExpectedF64(err) => write!(f, "Expected JSON f64: {}", err),
            Error::ExpectedI64(err) => write!(f, "Expected JSON i64: {}", err),
            Error::ExpectedObject(err) => write!(f, "Expected JSON object: {}", err),
            Error::ExpectedString(err) => write!(f, "Expected JSON string: {}", err),
            Error::Json(ref err) => write!(f, "JSON parse error: {}", err),
            Error::MalformedPoint(err) => write!(f, "Malformed point: {}", err),
            Error::MalformedSpectrum(err) => write!(f, "Malformed spectrum: {}", err),
            Error::MalformedVector(err) => write!(f, "Malformed vector: {}", err),
            Error::MissingKey(err) => write!(f, "Missing key: {}", err),
            Error::MissingReference { typ, name } => write!(f, "Referenced {} with name '{}' not found.", typ, name),
            Error::Texture(ref err) => write!(f, "Texture error: {}", err)
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

// TODO: use proper error handling here, i.e. Result

/// Parse the scene description from a JSON formatted string.
pub fn parse_scene(json: &str) -> Result<(Scene, HashMap<String, View>)> {
	let data: Value = try!(serde_json::from_str(json));

    let cameras = try!(data.pointer("/cameras").ok_or(Error::MissingKey("cameras")));
    let views = try!(data.pointer("/views").ok_or(Error::MissingKey("views")));
    let objects = try!(data.pointer("/objects").ok_or(Error::MissingKey("objects")));
    let materials = try!(data.pointer("/materials").ok_or(Error::MissingKey("materials")));
    let lights = try!(data.pointer("/lights").ok_or(Error::MissingKey("lights")));

    let cameras = try!(parse_cameras(cameras));
    let views = try!(parse_views(views, &cameras));
    let materials = try!(parse_materials(materials));
    let objects = try!(parse_objects(objects, &materials));
    let lights = try!(parse_lights(lights));

    let mut scene = Scene::new(objects);
    for light in lights {
        scene.add_light(light);
    }
    Ok((scene, views))
}

/// Parse a map of camera names to camera objects.
///
/// Structure:
/// {
///     "cameras": {
///         "main": {
///             ...
///         }
///     }
/// }
fn parse_cameras(data: &Value) -> Result<HashMap<String, Arc<Camera + Sync + Send>>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("cameras")));

    let mut cameras = HashMap::new();
    for (name, value) in data.iter() {
        let camera = try!(parse_camera(value));
        cameras.insert(name.clone(), camera);
    }
    Ok(cameras)
}

fn parse_camera(data: &Value) -> Result<Arc<Camera + Sync + Send>> {
    let transform = try!(data.pointer("/transform").ok_or(Error::MissingKey("transform")));
    let transform = try!(parse_transform(transform));

    let width = try!(data.pointer("/width").ok_or(Error::MissingKey("width")));
    let width = try!(try_get_u64(width, "width"));

    let height = try!(data.pointer("/height").ok_or(Error::MissingKey("height")));
    let height = try!(try_get_u64(height, "height"));

    let fov = try!(data.pointer("/fov").ok_or(Error::MissingKey("fov")));
    let fov = try!(try_get_f64(fov, "fov"));

    let near = try!(data.pointer("/near").ok_or(Error::MissingKey("near")));
    let near = try!(try_get_f64(near, "near"));

    let far = try!(data.pointer("/far").ok_or(Error::MissingKey("far")));
    let far = try!(try_get_f64(far, "far"));

    let camera_type = try!(data.pointer("/type").ok_or(Error::MissingKey("type")));
    let camera_type = try!(try_get_string(camera_type, "type"));

    match camera_type {
        "Perspective" => Ok(Arc::new(PerspectiveCamera::new(transform, 
                                                         width as u32, 
                                                         height as u32, 
                                                         fov.to_radians(), 
                                                         near, 
                                                         far)) as Arc<Camera + Sync + Send>),
        _ => panic!("Unrecognised camera type: {}", camera_type)
    }
}

fn parse_views(data: &Value, cameras: &HashMap<String, Arc<Camera + Sync + Send>>) -> Result<HashMap<String, View>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("views")));

    let mut views = HashMap::new();
    for (name, value) in data.iter() {
        let view = try!(parse_view(value, cameras));
        views.insert(name.clone(), view);
    }
    Ok(views)
}

fn parse_view(data: &Value, cameras: &HashMap<String, Arc<Camera + Sync + Send>>) -> Result<View> {
    let camera = try!(data.pointer("/camera").ok_or(Error::MissingKey("camera")));
    let camera = try!(camera.as_str().ok_or(Error::ExpectedString("camera")));
    // let camera = try!(cameras.get(camera).ok_or(Error::MissingReference(("Camera", camera))));
    let camera = cameras.get(camera).unwrap();

    let samples = try!(data.pointer("/samples").ok_or(Error::MissingKey("samples")));
    let samples = try!(try_get_i64(samples, "samples"));

    let depth = try!(data.pointer("/depth").ok_or(Error::MissingKey("depth")));
    let depth = try!(try_get_i64(depth, "depth"));

    let integrator = try!(data.pointer("/integrator").ok_or(Error::MissingKey("integrator")));
    let integrator = try!(try_get_string(integrator, "integrator"));

    let integrator = match integrator {
        "Path" => Box::new(PathTraced::new(depth as i32)) as Box<Integrator + Sync + Send>,
        "Whitted" => Box::new(Whitted::new(depth as i32)) as Box<Integrator + Sync + Send>,
        _ => panic!("Unrecognised integrator: {}", integrator)
    };

    let renderer = try!(data.pointer("/renderer").ok_or(Error::MissingKey("renderer")));
    let renderer = try!(renderer.as_str().ok_or(Error::ExpectedString("renderer")));
    let renderer = match renderer {
        "Standard" => Arc::new(StandardRenderer::new(integrator)) as Arc<Renderer + Sync + Send>,
        _ => panic!("Unrecognised renderer: {}", renderer)
    };

    Ok(View::new(camera.clone(), samples as u32, depth as i32, renderer))
}

fn parse_materials(data: &Value) -> Result<HashMap<String, Arc<Material + Sync + Send>>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("materials")));
    let mut materials = HashMap::new();
    for (name, value) in data.iter() {
        let material = try!(parse_material(value));
        materials.insert(name.clone(), material);
    }
    Ok(materials)
}

fn parse_material(data: &Value) -> Result<Arc<Material + Sync + Send>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("material")));

    let material_type = try!(data.get("type").ok_or(Error::MissingKey("type")));
    let material_type = try!(try_get_string(material_type, "type"));
    match material_type {
        "Glass" => Ok(Arc::new(GlassMaterial) as Arc<Material + Sync + Send>),
        "Mirror" => Ok(Arc::new(MirrorMaterial) as Arc<Material + Sync + Send>),
        "Diffuse" => Ok(Arc::new(try!(parse_diffuse_material(data))) as Arc<Material + Sync + Send>),
        _ => panic!("Unrecognised material type: {}", material_type)
    }
}

fn parse_diffuse_material(data: &Map<String, Value>) -> Result<DiffuseMaterial> {
    let texture = try!(data.get("texture").ok_or(Error::MissingKey("texture")));
    let texture = try!(parse_texture(&texture));
    Ok(DiffuseMaterial::new(texture))
}

fn parse_texture(data: &Value) -> Result<Box<Texture + Sync + Send>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("texture")));

    let texture_type = try!(data.get("type").ok_or(Error::MissingKey("type")));
    let texture_type = try!(try_get_string(texture_type, "type"));
    match texture_type {
        "Constant" => Ok(Box::new(try!(parse_constant_texture(data))) as Box<Texture + Sync + Send>),
        "Image" => Ok(Box::new(try!(parse_image_texture(data))) as Box<Texture + Sync + Send>),
        _ => panic!("Unrecognised texture type: {}", texture_type)
    }
}

fn parse_constant_texture(data: &Map<String, Value>) -> Result<ConstantTexture> {
    let colour = try!(data.get("colour").ok_or(Error::MissingKey("colour")));
    let colour = try!(parse_spectrum(colour));
    Ok(ConstantTexture::new(colour))
}

fn parse_image_texture(data: &Map<String, Value>) -> Result<ImageTexture> {
    let filename = try!(data.get("filename").ok_or(Error::MissingKey("filename")));
    let filename = try!(try_get_string(filename, "filename"));
    // TODO: use a centralised location for loading/storing assets
    let image = try!(image::open(&Path::new(filename)));
    let image = Arc::new(image.to_rgb());
    Ok(ImageTexture::new(image.clone()))
}

fn parse_objects(data: &Value, materials: &HashMap<String, Arc<Material + Sync + Send>>) -> Result<Vec<Arc<SceneNode>>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("objects")));

    // let mut objects = HashMap::new();
    let mut objects = Vec::new();
    for (name, value) in data.iter() {
        let object = Arc::new(try!(parse_object(value, materials)));
        // objects.insert(*name, object);
        objects.push(object);
    }
    Ok(objects)
}

fn parse_object(data: &Value, materials: &HashMap<String, Arc<Material + Sync + Send>>) -> Result<SceneNode> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("object")));

    let material = try!(data.get("material").ok_or(Error::MissingKey("material")));
    let material = try!(try_get_string(material, "material"));
    let material = materials.get(material)
        .expect(&format!("No Material found with name: {}", material));

    let transform = try!(data.get("transform").ok_or(Error::MissingKey("transform")));
    let transform = try!(parse_transform(transform));

    let Intersectable = try!(data.get("Intersectable").ok_or(Error::MissingKey("Intersectable")));
    let Intersectable = try!(try_get_string(Intersectable, "Intersectable"));
    let (Intersectable, aabb) = match Intersectable {
        "Cuboid" => try!(parse_cuboid(data, &transform)),
        "Ball" => try!(parse_ball(data, &transform)),
        _ => panic!("Unrecognised Intersectable: {}", Intersectable)
    };

    Ok(SceneNode::new(transform, material.clone(), Intersectable, aabb))
}

fn parse_cuboid(data: &Map<String, Value>, transform: &Isometry3<Scalar>) -> Result<(Intersectable, AABB3<Scalar>)> {
    let extents = try!(data.get("extents").ok_or(Error::MissingKey("extents")));
    let extents = try!(parse_vector(extents));

    let cuboid = Cuboid::new(extents);
    let aabb = cuboid.aabb(transform);
    Ok((Box::new(cuboid) as Box<RayCast<Point, Isometry3<Scalar>> + Sync + Send>, aabb))
}

fn parse_ball(data: &Map<String, Value>, transform: &Isometry3<Scalar>) -> Result<(Intersectable, AABB3<Scalar>)> {
    let radius = try!(data.get("radius").ok_or(Error::MissingKey("radius")));
    let radius = try!(try_get_f64(radius, "radius"));

    let ball = Ball::new(radius);
    let aabb = ball.aabb(transform);
    Ok((Box::new(ball) as Box<RayCast<Point, Isometry3<Scalar>> + Sync + Send>, aabb))
}

fn parse_lights(data: &Value) -> Result<Vec<Box<Light + Sync + Send>>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("lights")));

    let mut lights = Vec::new();
    for (_, value) in data.iter() {
        let light = try!(parse_light(value));
        lights.push(light);
    }
    Ok(lights)
}

fn parse_light(data: &Value) -> Result<Box<Light + Sync + Send>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("light")));

    let light_type = try!(data.get("type").ok_or(Error::MissingKey("type")));
    let light_type = try!(try_get_string(light_type, "type"));

    let colour = try!(data.get("colour").ok_or(Error::MissingKey("colour")));
    let colour = try!(parse_spectrum(colour));

    match light_type {
        "Point" => Ok(Box::new(try!(parse_point_light(data, colour))) as Box<Light + Sync + Send>),
        _ => panic!("Unrecognised light type: {}", light_type)
    }
}

fn parse_point_light(data: &Map<String, Value>, colour: Spectrum) -> Result<PointLight> {
    let position = try!(data.get("position").ok_or(Error::MissingKey("position")));
    let position = try!(parse_point(position));

    let radius = try!(data.get("radius").ok_or(Error::MissingKey("radius")));
    let radius = try!(try_get_f64(radius, "radius"));

    Ok(PointLight::new(1.0, colour, position, radius))
}

fn parse_vector(data: &Value) -> Result<Vector> {
    let data = try!(data.as_array().ok_or(Error::ExpectedArray("vector")));
    if data.len() != 3 {
        return Err(Error::MalformedVector("Array of three elements expected."));
    }

    let x = try!(try_get_f64(&data[0], "x"));
    let y = try!(try_get_f64(&data[1], "y"));
    let z = try!(try_get_f64(&data[2], "z"));

    Ok(Vector::new(x, y, z))
}

fn parse_point(data: &Value) -> Result<Point> {
    let data = try!(data.as_array().ok_or(Error::ExpectedArray("point")));
    if data.len() != 3 {
        return Err(Error::MalformedPoint("Array of three elements expected."));
    }

    let x = try!(try_get_f64(&data[0], "x"));
    let y = try!(try_get_f64(&data[1], "y"));
    let z = try!(try_get_f64(&data[2], "z"));

    Ok(Point::new(x, y, z))
}

fn parse_transform(data: &Value) -> Result<Isometry3<Scalar>> {
    let data = try!(data.as_object().ok_or(Error::ExpectedObject("transform")));

    let position = try!(data.get("position").ok_or(Error::MissingKey("position")));
    let position = try!(parse_vector(position));

    let rotation = match data.get("rotation") {
        Some(rot) => try!(parse_vector(rot)),
        None => na::zero()
    };

    Ok(Isometry3::new(position, rotation))
}

fn parse_spectrum(data: &Value) -> Result<Spectrum> {
    let data = try!(data.as_array().ok_or(Error::ExpectedArray("spectrum")));
    if data.len() != 3 {
        return Err(Error::MalformedSpectrum("Array of three elements expected"));
    }

    let r = try!(try_get_f64(&data[0], "r"));
    let g = try!(try_get_f64(&data[1], "g"));
    let b = try!(try_get_f64(&data[2], "b"));

    Ok(Spectrum::new(r, g, b))
}

fn try_get_string<'a>(value: &'a Value, key: &'static str) -> Result<&'a str> {
    value.as_str().ok_or(Error::ExpectedString(key))
}

fn try_get_object<'a>(value: &'a Value, key: &'static str) -> Result<&'a Map<String, Value>> {
    value.as_object().ok_or(Error::ExpectedObject(key))
}

fn try_get_u64(value: &Value, key: &'static str) -> Result<u64> {
    value.as_u64().ok_or(Error::ExpectedU64(key))
}

fn try_get_f64(value: &Value, key: &'static str) -> Result<f64> {
    value.as_f64().ok_or(Error::ExpectedF64(key))
}

fn try_get_i64(value: &Value, key: &'static str) -> Result<i64> {
    value.as_i64().ok_or(Error::ExpectedI64(key))
}
