
use std;
use std::sync::Arc;
use uuid::Uuid;

use na;
use na::{Iso3, Pnt3, Vec3};
use ncollide::bounding_volume::{AABB3, HasAABB};
use ncollide::partitioning::{DBVT};
use ncollide::ray::{RayCast, Ray3, RayInterferencesCollector};

use light::{Light};
use spectrum::{Spectrum};
use surface::{SurfaceIntegrator};

pub struct SceneNode {
    pub uuid: Uuid,
    pub transform: Iso3<f64>,
    pub surface: Box<SurfaceIntegrator>,
    pub geom: Box<RayCast<Pnt3<f64>, Iso3<f64>>>,
    pub aabb: AABB3<f64>,
    pub solid: bool
}

impl SceneNode {
    pub fn new<S: 'static + SurfaceIntegrator, N: 'static + RayCast<Pnt3<f64>, Iso3<f64>> + HasAABB<Pnt3<f64>, Iso3<f64>>>(
        transform: Iso3<f64>,
        surface: Box<S>,
        geom: Box<N>,
        solid: bool) -> SceneNode {
        SceneNode {
            uuid : Uuid::new_v4(),
            transform : transform,
            surface : surface as Box<SurfaceIntegrator>,
            aabb : geom.aabb(&transform),
            geom : geom as Box<RayCast<Pnt3<f64>, Iso3<f64>>>,
            solid : solid
        }
    }
}

pub struct Scene {
	pub lights: Vec<Box<Light>>,
    world: DBVT<Pnt3<f64>, Arc<SceneNode>, AABB3<f64>>
}

/// Get the nearest node and surface info at the intersection
/// point intersected by the given ray.
fn get_nearest<'a>(ray: &Ray3<f64>, nodes: &'a [Arc<SceneNode>]) -> Option<(&'a SceneNode, f64, Vec3<f64>)> {
    // TODO: ability to ignore intersections with certain SceneNodes
    let mut nearest_node: Option<&SceneNode> = None;
    let mut nearest_toi = std::f64::MAX;
    let mut nearest_normal = na::zero();
    for node in nodes {
        match node.geom.toi_and_normal_with_transform_and_ray(&node.transform, ray, true) {
            Some(isect) => {
                // check toi is greater than zero to rule out intersection
                // with the node whose surface we're casting a ray from
                // Note: this is not 100% reliable I don't think
                if isect.toi > 0.0 && isect.toi < nearest_toi {
                    nearest_node = Some(node);
                    nearest_toi = isect.toi;
                    nearest_normal = isect.normal;
                }
            },
            _ => {}
        }
    }
    if nearest_node.is_some() {
        Some((nearest_node.unwrap(), nearest_toi, nearest_normal))
    } else {
        None
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            lights: Vec::new(),
            world: DBVT::new()
        }
    }

    pub fn add_node(&mut self, node: Arc<SceneNode>) {
        self.world.insert_new(node.clone(), node.aabb.clone());
    }

    pub fn add_light(&mut self, light: Box<Light>) {
        self.lights.push(light);
    }

    pub fn trace(&self, ray: &Ray3<f64>, depth: u32) -> Spectrum {
        let mut colour: Spectrum = na::zero();
        let mut intersections = Vec::new();
        {
            // we define a scope here so that visitor (which takes a mutable
            // borrow of the intersections vector) is dropped before we need
            // to take another borrow of the intersections vector later on
            let mut visitor = RayInterferencesCollector::new(ray, &mut intersections);
            self.world.visit(&mut visitor);
        }

        // cast a ray towards the light to see if this point
        // should receive any luminance from it
        // update: this is not a great idea for the naive
        // lighting method in this case as points further away
        // from the light on a sphere might intersect the sphere
        // itself and interfere with the calculations
        // when geometry is used to represent a light this
        // won't be as much of an issue along with using
        // ray differentials
        match get_nearest(ray, &intersections) {
            Some((ref node, toi, normal)) => {
                // TODO: this should really trace a ray from
                // the point to the light to see if it visible
                // from the light and that there is no object
                // obscuring it (only relevant for lights other
                // than directional or ambient).
                let p = ray.orig + ray.dir * toi;

                // let light_ray = light.
                // TODO: incorporate colour from the object itself
                // colour of object is set to all 1 for now
                // TODO: attenuate amount of light energy
                // reflected from the surface
                let c = node.surface.sample(&ray.dir,
                                            &p,
                                            &normal,
                                            &na::one(), // surface colour of 1 for now
                                            self,
                                            depth);
                colour = colour + c;
            },
            None => {}
        }
        colour
    }
}