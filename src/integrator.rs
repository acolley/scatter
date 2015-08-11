
use na;
use na::{Vec3};

use bxdf::{BxDFType, BSDF_ALL, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION};
use light::{Light};
use rand::{Rng};
use ray::{Ray};
use renderer::{Renderer};
use scene::{Intersection, Scene};
use spectrum::{Spectrum};

// maximum depth to perform actual
// sampling techniques in path tracing
const SAMPLE_DEPTH: i32 = 3;

#[inline]
fn sample_light(light: &Box<Light + Send + Sync>,
                wo: &Vec3<f64>,
                isect: &Intersection,
                scene: &Scene,
                flags: BxDFType) -> Spectrum {
    let (li, wi) = light.sample(&isect.point);
    if li == na::zero() {
        return na::zero();
    }
    let bsdf = &isect.bsdf;
    let f = bsdf.f(wo, &wi, flags);
    if f == na::zero() || light.shadow(&isect.point, scene) {
        na::zero()
    } else {
        f * li * na::dot(&isect.normal, &wi)
    }
}

pub fn sample_one_light<R>(
    wo: &Vec3<f64>,
    isect: &Intersection,
    scene: &Scene,
    rng: &mut R) -> Spectrum
where R: Rng {
    let nlights = scene.lights.len();
    if nlights == 0 {
        return na::zero();
    }
    let bsdf = &isect.bsdf;
    let light = rng.choose(&scene.lights).expect("Light could not be chosen");
    let l = sample_light(light, wo, isect, scene, BSDF_ALL - BSDF_SPECULAR) * nlights as f64;
    l
}

/// Integrate over all lights computing
/// direct lighting at a surface point
/// and sampling the BSDF at the intersection.
pub fn sample_all_lights(wo: &Vec3<f64>, 
                         isect: &Intersection,
                         scene: &Scene) -> Spectrum {
    let bsdf = &isect.bsdf;
    let mut l = na::zero();
    for light in &scene.lights {
        l = l + sample_light(light, wo, isect, scene, BSDF_ALL - BSDF_SPECULAR);
    }
    l
}

/// Find the specular reflection component at a surface point.
pub fn specular_reflect<R, T>(
    ray: &Ray,
    isect: &Intersection, 
    scene: &Scene,
    renderer: &T,
    rng: &mut R) -> Spectrum
where R: Rng,
      T: Renderer {
    let wo = -(*ray.dir());
    let n = &isect.normal;
    let bsdf = &isect.bsdf;
    let (f, wi, pdf) = bsdf.sample_f(&wo, rng, BSDF_REFLECTION | BSDF_SPECULAR);
    if pdf > 0.0 && f != na::zero() && na::dot(&wi, n) != 0.0 {
        // move the ray origin forward by a small amount in its direction
        // to avoid intersection with the surface we just came from
        let ray = Ray::new_with_depth(isect.point + wi * 0.000000000001, wi, ray.depth + 1);
        let li = renderer.render(&ray, scene, rng);
        let l = f * li * (na::dot(&wi, n).abs() / pdf);
        l
    } else {
        na::zero()
    }
}

/// Find the specular transmission component at a surface point.
pub fn specular_transmit<R, T>(
    ray: &Ray,
    isect: &Intersection, 
    scene: &Scene,
    renderer: &T,
    rng: &mut R) -> Spectrum
where R: Rng,
      T: Renderer {
    let wo = -(*ray.dir());
    let n = &isect.normal;
    let bsdf = &isect.bsdf;
    let (f, wi, pdf) = bsdf.sample_f(&wo, rng, BSDF_TRANSMISSION | BSDF_SPECULAR);
    if pdf > 0.0 && f != na::zero() && na::dot(&wi, n) != 0.0 {
        // move the ray origin forward by a small amount in its direction
        // to avoid intersection with the surface we just came from
        let ray = Ray::new_with_depth(isect.point + wi * 0.000000000001, wi, ray.depth + 1);
        let li = renderer.render(&ray, scene, rng);
        let l = f * li * (na::dot(&wi, n).abs() / pdf);
        l
    } else {
        na::zero()
    }
}

pub trait Integrator {
    fn integrate<R, T>(
        &self, 
        ray: &Ray, 
        isect: &Intersection, 
        scene: &Scene, 
        renderer: &T,
        rng: &mut R) -> Spectrum
    where R: Rng,
          T: Renderer;
}

pub struct Whitted {
    depth: i32
}

impl Whitted {
    pub fn new(depth: i32) -> Whitted {
        Whitted {
            depth : depth
        }
    }
}

impl Integrator for Whitted {
    fn integrate<R, T>(
        &self, 
        ray: &Ray, 
        isect: &Intersection, 
        scene: &Scene, 
        renderer: &T,
        rng: &mut R) -> Spectrum
    where R: Rng,
          T: Renderer {
        let wo = -(*ray.dir());
        let mut l = sample_all_lights(&wo, &isect, scene);

        if ray.depth < self.depth {
            l = l + specular_reflect(ray, &isect, scene, renderer, rng);
            l = l + specular_transmit(ray, &isect, scene, renderer, rng);
        }
        l
    }
}

pub struct PathTraced {
    depth: i32
}

impl PathTraced {
    pub fn new(depth: i32) -> PathTraced {
        PathTraced {
            depth : depth
        }
    }
}

impl Integrator for PathTraced {
    fn integrate<R, T>(
        &self,
        ray: &Ray,
        isect: &Intersection,
        scene: &Scene,
        renderer: &T,
        rng: &mut R) -> Spectrum
    where R: Rng,
          T: Renderer {
        let throughput = Vec3::new(1.0, 1.0, 1.0);
        let mut specular_bounce = false;
        let mut l = na::zero();
        for bounce in 0..self.depth {
            // add emitted light at path vertex
            // if bounce == 0 || specular_bounce {
            //     L = L + throughput * 
            // }
            // TODO: should be currently sampled path's Intersection's BSDF
            let bsdf = &isect.bsdf;
            let wo = -(*ray.dir());
            if bounce < SAMPLE_DEPTH {
                // TODO: this should perform proper sampling
                // using Monte Carlo techniques, currently it's
                // exactly the same as the other branch
                l = l + throughput * sample_one_light(&wo, isect, scene, rng);
            } else {
                l = l + throughput * sample_one_light(&wo, isect, scene, rng);
            }

            // sample BSDF to get next direction for path
            // TODO: return the type of path that was sampled
            let (f, wi, _) = bsdf.sample_f(&wo, rng, BSDF_ALL);

        }
        l
    }
}