
use std;
use std::sync::Arc;

use na;
use na::{Iso3, Pnt3, Vec3};
use ncollide::bounding_volume::{BoundingSphere3, HasBoundingSphere};
use ncollide::partitioning::{DBVT};
use ncollide::ray::{RayCast, Ray3, RayIntersection, RayInterferencesCollector};

use light::{Light};
use spectrum::{Spectrum};
use surface::{SurfaceIntegrator};

pub struct SceneNode {
    pub transform: Iso3<f64>,
    pub surface: Box<SurfaceIntegrator>,
    pub geom: Box<RayCast<Pnt3<f64>, Iso3<f64>>>,
    pub bsphere: BoundingSphere3<f64>
}

impl SceneNode {
    pub fn new<S: 'static + SurfaceIntegrator, N: 'static + RayCast<Pnt3<f64>, Iso3<f64>> + HasBoundingSphere<Pnt3<f64>, Iso3<f64>>>(
        transform: Iso3<f64>,
        surface: Box<S>,
        geom: Box<N>) -> SceneNode {
        SceneNode {
            transform : transform,
            surface : surface as Box<SurfaceIntegrator>,
            bsphere : geom.bounding_sphere(&transform),
            geom : geom as Box<RayCast<Pnt3<f64>, Iso3<f64>>>
        }
    }
}

pub struct Scene {
	pub lights: Vec<Box<Light>>,
    world: DBVT<Pnt3<f64>, Arc<SceneNode>, BoundingSphere3<f64>>
}

/// Get the nearest node and surface info of the intersection
/// point intersected by the given ray.
fn get_nearest<'a>(ray: &Ray3<f64>, nodes: &'a [Arc<SceneNode>]) -> Option<(&'a SceneNode, f64, Vec3<f64>)> {
    // TODO: ability to ignore intersections with certain SceneNodes
    let mut nearest_node: Option<&SceneNode> = None;
    let mut nearest_toi = std::f64::MAX;
    let mut nearest_normal = na::zero();
    for node in nodes {
        match node.geom.toi_and_normal_with_transform_and_ray(&node.transform, ray, true) {
            Some(isect) => {
                if isect.toi < nearest_toi {
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
        self.world.insert_new(node.clone(), node.bsphere.clone());
    }

    pub fn add_light(&mut self, light: Box<Light>) {
        self.lights.push(light);
    }

    pub fn trace(&self, ray: &Ray3<f64>, depth: isize) -> Spectrum {
        let mut colour: Spectrum = na::zero();
        let mut intersections = Vec::new();
        {
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
                let c = node.surface.sample(&ray.dir,
                                            &p,
                                            &normal,
                                            &na::one(),
                                            self,
                                            depth);
                colour = colour + c;
            },
            None => {}
        }
        colour
    }
}