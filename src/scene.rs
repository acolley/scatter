
use std::sync::Arc;

use na;
use na::{Iso3, Pnt3, Vec3};
use ncollide::bounding_volume::{BoundingSphere3, HasBoundingSphere};
use ncollide::partitioning::{DBVT};
use ncollide::ray::{RayCast, Ray3, RayInterferencesCollector};

use light::{Light};
use spectrum::{Spectrum};
use surface::{Diffuse, SurfaceIntegrator};

pub struct SceneNode {
    pub transform: Iso3<f64>,
    pub geom: Box<RayCast<Pnt3<f64>, Iso3<f64>>>,
    pub bsphere: BoundingSphere3<f64>
}

impl SceneNode {
    pub fn new<N: 'static + RayCast<Pnt3<f64>, Iso3<f64>> + HasBoundingSphere<Pnt3<f64>, Iso3<f64>>>(
        transform: Iso3<f64>,
        geom: Box<N>) -> SceneNode {
        SceneNode {
            transform : transform,
            bsphere : geom.bounding_sphere(&transform),
            geom : geom as Box<RayCast<Pnt3<f64>, Iso3<f64>>>
        }
    }
}

pub struct Scene {
	lights: Vec<Box<Light>>,
    world: DBVT<Pnt3<f64>, Arc<SceneNode>, BoundingSphere3<f64>>
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
        let surface = Diffuse;
        let mut colour: Spectrum = na::zero();
        let mut intersections = Vec::new();
        {
            let mut visitor = RayInterferencesCollector::new(ray, &mut intersections);
            self.world.visit(&mut visitor);
        }

        if intersections.len() > 0 {
            let node = unsafe { intersections.get_unchecked(0) };
            match node.geom.toi_and_normal_with_transform_and_ray(&node.transform, ray, true) {
                Some(isect) => {
                    // TODO: this should really trace a ray from
                    // the point to the light to see if it visible
                    // from the light and that there is no object
                    // obscuring it (only relevant for lights other
                    // than directional or ambient).
                    let p = ray.orig + ray.dir * isect.toi;

                    // cast a ray towards the light to see if this point
                    // should receive any luminance from it
                    // update: this is not a great idea for the naive
                    // lighting method in this case as points further away
                    // from the light on a sphere might intersect the sphere
                    // itself and interfere with the calculations
                    // when geometry is used to represent a light this
                    // won't be as much of an issue along with using
                    // ray differentials

                    // let light_ray = light.
                    // TODO: incorporate colour from the object itself
                    // colour of object is set to all 1 for now
                    let c = surface.sample(&p, &isect.normal, &na::one(), &self.lights);
                    colour = colour + c;
                },
                None => {}
            }
        }
        colour
    }
}