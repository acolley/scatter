
use std;
use std::sync::Arc;
use uuid::Uuid;

use na;
use na::{Iso3, Pnt2, Pnt3, Vec3};
use ncollide::bounding_volume::{AABB3, HasAABB};
use ncollide::partitioning::{BVT};
use ncollide::ray::{RayCast, RayInterferencesCollector};

use bxdf::{BSDF};
use light::{Light};
use material::{Material};
use ray::{Ray};

/// Structure representing an object in the 
/// Scene that can be shaded.
pub struct SceneNode {
    pub uuid: Uuid,
    pub transform: Iso3<f64>,
    pub material: Arc<Material + Sync + Send>,
    pub geom: Box<RayCast<Pnt3<f64>, Iso3<f64>> + Sync + Send>,
    pub aabb: AABB3<f64>
}

/// Structure storing information about an
/// object that was intersected in the Scene.
pub struct Intersection {
    pub point: Pnt3<f64>,
    pub normal: Vec3<f64>,
    pub bsdf: BSDF
}

impl Intersection {
    pub fn new(p: Pnt3<f64>, n: Vec3<f64>, bsdf: BSDF) -> Intersection {
        Intersection {
            point : p,
            normal : n,
            bsdf : bsdf
        }
    }
}

impl SceneNode {
    pub fn new<M: 'static + Material + Sync + Send, N: 'static + RayCast<Pnt3<f64>, Iso3<f64>> + HasAABB<Pnt3<f64>, Iso3<f64>> + Sync + Send>(
        transform: Iso3<f64>,
        material: Arc<M>,
        geom: Box<N>) -> SceneNode {
        SceneNode {
            uuid : Uuid::new_v4(),
            transform : transform,
            material : material as Arc<Material + Sync + Send>,
            aabb : geom.aabb(&transform),
            geom : geom as Box<RayCast<Pnt3<f64>, Iso3<f64>> + Sync + Send>
        }
    }
}

pub struct Scene {
	pub lights: Vec<Box<Light + Sync + Send>>,
    world: BVT<Arc<SceneNode>, AABB3<f64>>
}

/// Get the nearest node and surface info at the intersection
/// point intersected by the given ray.
fn get_nearest<'a>(ray: &Ray, nodes: &'a [Arc<SceneNode>]) -> Option<(&'a SceneNode, f64, Vec3<f64>, Pnt2<f64>)> {
    let mut nearest_node: Option<&SceneNode> = None;
    let mut nearest_toi = std::f64::MAX;
    let mut nearest_normal = na::zero();
    let mut nearest_uvs = Pnt2::new(0.0, 0.0);
    for node in nodes {
        match node.geom.toi_and_normal_and_uv_with_transform_and_ray(&node.transform, &ray.ray, false) {
            Some(isect) => {
                // check toi is greater than zero to rule out intersection
                // with the node whose surface we're casting a ray from
                // Note: this is not 100% reliable I don't think
                // this in tandem with code in renderer for casting reflection
                // and transmission rays slightly off the point on the surface
                // they came from should hopefully prevent artifacts
                if isect.toi > 0.0 && isect.toi < nearest_toi {
                    nearest_node = Some(node);
                    nearest_toi = isect.toi;
                    nearest_normal = isect.normal;
                    // TODO: handle the case where uvs are not present
                    nearest_uvs = isect.uvs.unwrap();
                }
            },
            _ => {}
        }
    }
    if nearest_node.is_some() {
        Some((nearest_node.unwrap(), nearest_toi, nearest_normal, nearest_uvs))
    } else {
        None
    }
}

impl Scene {
    pub fn new(nodes: Vec<Arc<SceneNode>>) -> Scene {
        let leaves = nodes.iter().map(|n| (n.clone(), n.aabb.clone())).collect();
        Scene {
            lights: Vec::new(),
            world: BVT::new_balanced(leaves)
        }
    }

    #[inline]
    pub fn add_light<T: 'static + Light + Sync + Send>(&mut self, light: Box<T>) {
        self.lights.push(light as Box<Light + Sync + Send>);
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        let mut intersections = Vec::new();
        {
            // we define a scope here so that visitor (which takes a mutable
            // borrow of the intersections vector) is dropped before we need
            // to take another borrow of the intersections vector later on
            let mut visitor = RayInterferencesCollector::new(&ray.ray, &mut intersections);
            self.world.visit(&mut visitor);
        }
        intersections.iter().any(|n| n.geom.intersects_with_transform_and_ray(&n.transform, &ray.ray))
    }

    pub fn intersections(&self, ray: &Ray) -> Vec<f64> {
        let mut intersections = Vec::new();
        {
            // we define a scope here so that visitor (which takes a mutable
            // borrow of the intersections vector) is dropped before we need
            // to take another borrow of the intersections vector later on
            let mut visitor = RayInterferencesCollector::new(&ray.ray, &mut intersections);
            self.world.visit(&mut visitor);
        }
        intersections.iter()
                     .map(|n| n.geom.toi_with_transform_and_ray(&n.transform, &ray.ray, true))
                     .filter(|x| x.is_some())
                     .map(|x| x.unwrap())
                     .filter(|&x| x > 0.0)
                     .collect()
    }

    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        let mut intersections = Vec::new();
        {
            // we define a scope here so that visitor (which takes a mutable
            // borrow of the intersections vector) is dropped before we need
            // to take another borrow of the intersections vector later on
            let mut visitor = RayInterferencesCollector::new(&ray.ray, &mut intersections);
            self.world.visit(&mut visitor);
        }

        match get_nearest(ray, &intersections) {
            Some((ref node, toi, normal, uvs)) => {
                let p = *ray.orig() + *ray.dir() * toi;
                Some(Intersection::new(p, normal, node.material.get_bsdf(&normal, &uvs)))
            },
            None => None
        }
    }
}