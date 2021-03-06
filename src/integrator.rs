
use na;

use bxdf::{BxDFType, BSDF_ALL, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION};
use light::Light;
use math::{Scalar, Vector};
use rand::{Rng, StdRng};
use ray::Ray;
use renderer::Renderer;
use scene::{Intersection, Scene};
use spectrum::Spectrum;

// maximum depth to perform actual
// sampling techniques in path tracing
const SAMPLE_DEPTH: i32 = 3;

fn luminance(c: &Spectrum) -> Scalar {
    c.x * 0.2126 + c.y * 0.7152 + c.z * 0.0722
}

#[inline]
fn sample_light(light: &Box<Light + Send + Sync>,
                wo: &Vector,
                isect: &Intersection,
                scene: &Scene,
                flags: BxDFType)
                -> Spectrum {
    let (li, wi) = light.sample(&isect.point);
    if li == na::zero() {
        return na::zero();
    }
    let bsdf = &isect.bsdf;
    let f = bsdf.f(wo, &wi, flags);
    if f == na::zero() || light.shadow(&isect.point, scene) {
        na::zero()
    } else {
        f.component_mul(&li) * na::dot(&isect.normal, &wi)
    }
}

pub fn sample_one_light(wo: &Vector,
                        isect: &Intersection,
                        scene: &Scene,
                        rng: &mut StdRng)
                        -> Spectrum {
    let nlights = scene.lights.len();
    if nlights == 0 {
        return na::zero();
    }
    let light = rng.choose(&scene.lights).expect("Light could not be chosen");
    sample_light(light, wo, isect, scene, BSDF_ALL - BSDF_SPECULAR) * nlights as f64
}

/// Integrate over all lights computing
/// direct lighting at a surface point
/// and sampling the BSDF at the intersection.
pub fn sample_all_lights(wo: &Vector, isect: &Intersection, scene: &Scene) -> Spectrum {
    scene.lights
        .iter()
        .map(|l| sample_light(l, wo, isect, scene, BSDF_ALL - BSDF_SPECULAR))
        .fold(na::zero(), |acc, c| acc + c)
}

/// Find the specular reflection component at a surface point.
pub fn specular_reflect(ray: &Ray,
                        isect: &Intersection,
                        scene: &Scene,
                        renderer: &Renderer,
                        rng: &mut StdRng)
                        -> Spectrum {
    let wo = -(*ray.dir());
    let n = &isect.normal;
    let bsdf = &isect.bsdf;
    let (f, wi, pdf, _) = bsdf.sample_f(&wo, rng, BSDF_REFLECTION | BSDF_SPECULAR);
    if pdf > 0.0 && f != na::zero() && na::dot(&wi, n) != 0.0 {
        // move the ray origin forward by a small amount in its direction
        // to avoid intersection with the surface we just came from
        let ray = Ray::new_with_depth(isect.point + wi * 0.000000000001, wi, ray.depth + 1);
        let li = renderer.render(&ray, scene, rng);
        f.component_mul(&li) * (na::dot(&wi, n).abs() / pdf)
    } else {
        na::zero()
    }
}

/// Find the specular transmission component at a surface point.
pub fn specular_transmit(ray: &Ray,
                         isect: &Intersection,
                         scene: &Scene,
                         renderer: &Renderer,
                         rng: &mut StdRng)
                         -> Spectrum {
    let wo = -(*ray.dir());
    let n = &isect.normal;
    let bsdf = &isect.bsdf;
    let (f, wi, pdf, _) = bsdf.sample_f(&wo, rng, BSDF_TRANSMISSION | BSDF_SPECULAR);
    if pdf > 0.0 && f != na::zero() && na::dot(&wi, n) != 0.0 {
        // move the ray origin forward by a small amount in its direction
        // to avoid intersection with the surface we just came from
        let ray = Ray::new_with_depth(isect.point + wi * 0.000000000001, wi, ray.depth + 1);
        let li = renderer.render(&ray, scene, rng);
        f.component_mul(&li) * (na::dot(&wi, n).abs() / pdf)
    } else {
        na::zero()
    }
}

pub trait Integrator {
    fn integrate(&self,
                 ray: &Ray,
                 isect: &Intersection,
                 scene: &Scene,
                 renderer: &Renderer,
                 rng: &mut StdRng)
                 -> Spectrum;
}

pub struct Whitted {
    depth: i32,
}

impl Whitted {
    pub fn new(depth: i32) -> Whitted {
        Whitted { depth: depth }
    }
}

impl Integrator for Whitted {
    fn integrate(&self,
                 ray: &Ray,
                 isect: &Intersection,
                 scene: &Scene,
                 renderer: &Renderer,
                 rng: &mut StdRng)
                 -> Spectrum {
        let wo = -(*ray.dir());
        let mut l = sample_all_lights(&wo, isect, scene);

        if ray.depth < self.depth {
            l = l + specular_reflect(ray, isect, scene, renderer, rng);
            l = l + specular_transmit(ray, isect, scene, renderer, rng);
        }
        l
    }
}

pub struct PathTraced {
    depth: i32,
}

impl PathTraced {
    pub fn new(depth: i32) -> PathTraced {
        PathTraced { depth: depth }
    }

    pub fn depth(&self) -> i32 {
        self.depth
    }
}

fn path_bounce(tracer: &PathTraced,
               ray: &Ray,
               isect: &Intersection,
               scene: &Scene,
               renderer: &Renderer,
               rng: &mut StdRng,
               bounce: i32,
               throughput: Spectrum,
               specular_bounce: bool)
               -> Spectrum {
    let mut l = na::zero();
    let bsdf = &isect.bsdf;
    let wo = -(*ray.dir());
    // TODO: add emitted light at path vertex
    // if bounce == 0 || specular_bounce {
    //     l = l + throughput *
    // }
    if bounce < SAMPLE_DEPTH {
        // TODO: this should perform proper sampling
        // using Monte Carlo techniques, currently it's
        // exactly the same as the other branch
        l = l + throughput.component_mul(&sample_one_light(&wo, isect, scene, rng));
    } else {
        l = l + throughput.component_mul(&sample_one_light(&wo, isect, scene, rng));
    }

    // sample BSDF to get next direction for path
    let (f, wi, pdf, flags) = bsdf.sample_f(&wo, rng, BSDF_ALL);
    if f == na::zero() || pdf == 0.0 {
        return l;
    }
    let flags = flags.unwrap();
    let specular_bounce = flags.intersects(BSDF_SPECULAR);
    let mut throughput = throughput.component_mul(&f) * na::dot(&wi, &isect.normal).abs() / pdf;
    let ray = Ray::new(isect.point + wi * 0.000000000001, wi);

    // possibly terminate the path using russian roulette
    if bounce > 3 {
        let continue_probability = f64::min(0.5, luminance(&throughput));
        if rng.next_f64() > continue_probability {
            return l;
        }
        throughput = throughput / continue_probability;
    }

    // Reached maximum depth so terminate path.
    if bounce == tracer.depth() {
        return l;
    }

    l +
    match scene.trace(&ray) {
        Some(isect) => {
            // TODO: take transmittance into account
            path_bounce(tracer,
                        &ray,
                        &isect,
                        scene,
                        renderer,
                        rng,
                        bounce + 1,
                        throughput,
                        specular_bounce)
        }
        None => {
            if specular_bounce {
                // TODO: get light from all lights
                // emitted in the incident direction
                // given by wi
            }
            na::zero()
        }
    }
}

impl Integrator for PathTraced {
    fn integrate(&self,
                 ray: &Ray,
                 isect: &Intersection,
                 scene: &Scene,
                 renderer: &Renderer,
                 rng: &mut StdRng)
                 -> Spectrum {
        path_bounce(self,
                    ray,
                    isect,
                    scene,
                    renderer,
                    rng,
                    0,
                    Vector::new(1.0, 1.0, 1.0),
                    false)
    }
}
