
use std;
use std::sync::Arc;
use uuid::Uuid;

use na;
use na::{Isometry3, Point2, Point3};
use ncollide::bounding_volume::AABB3;
use ncollide::partitioning::BVT;
use ncollide::query::{RayCast, RayInterferencesCollector};

use bxdf::BSDF;
use light::Light;
use material::Material;
use math::{Normal, Point, Scalar, Vector};
use ray::Ray;

/// Structure representing an object in the
/// Scene that can be shaded.
pub struct SceneNode {
    pub uuid: Uuid,
    pub transform: Isometry3<Scalar>,
    pub material: Arc<Material + Sync + Send>,
    pub geom: Box<RayCast<Point, Isometry3<Scalar>> + Sync + Send>,
    pub aabb: AABB3<Scalar>,
}

/// Structure storing information about an
/// object that was intersected in the Scene.
pub struct Intersection {
    pub point: Point,
    pub normal: Normal,
    pub bsdf: BSDF,
}

impl Intersection {
    pub fn new(p: Point, n: Normal, bsdf: BSDF) -> Intersection {
        Intersection {
            point: p,
            normal: n,
            bsdf: bsdf,
        }
    }
}

impl SceneNode {
    pub fn new(transform: Isometry3<Scalar>,
               material: Arc<Material + Sync + Send>,
               geom: Box<RayCast<Point, Isometry3<Scalar>> + Sync + Send>,
               aabb: AABB3<Scalar>)
               -> SceneNode {
        SceneNode {
            uuid: Uuid::new_v4(),
            transform: transform,
            material: material,
            aabb: aabb,
            geom: geom,
        }
    }
}

pub struct Scene {
    pub lights: Vec<Box<Light + Sync + Send>>,
    world: BVT<Arc<SceneNode>, AABB3<Scalar>>,
}

/// Get the nearest node and surface info at the intersection
/// point intersected by the given ray.
fn get_nearest<'a>(ray: &Ray,
                   nodes: &'a [Arc<SceneNode>])
                   -> Option<(&'a SceneNode, f64, Vector, Option<Point2<f64>>)> {
    let mut nearest_node: Option<&SceneNode> = None;
    let mut nearest_toi = std::f64::MAX;
    let mut nearest_normal = na::zero();
    let mut nearest_uvs = None;
    for node in nodes {
        if let Some(isect) = node.geom
            .toi_and_normal_and_uv_with_ray(&node.transform, &ray.ray, false) {
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
                nearest_uvs = isect.uvs;
            }
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
            world: BVT::new_balanced(leaves),
        }
    }

    #[inline]
    pub fn add_light(&mut self, light: Box<Light + Sync + Send>) {
        self.lights.push(light);
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
        intersections.iter().any(|n| n.geom.intersects_ray(&n.transform, &ray.ray))
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
            .map(|n| n.geom.toi_with_ray(&n.transform, &ray.ray, true))
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
            Some((node, toi, normal, uvs)) => {
                let p = *ray.orig() + *ray.dir() * toi;
                Some(Intersection::new(p, normal, node.material.get_bsdf(&normal, &uvs)))
            }
            None => None,
        }
    }
}
