
use na;
use na::{Pnt3, Vec3};
use ncollide::ray::{Ray3};

use bxdf::{BSDF_ALL, BSDF_REFLECTION, BSDF_TRANSMISSION};
use ray::{Ray};
use scene::{Intersection, Scene};
use spectrum::{Spectrum};

/// Integrate over direct lighting at a surface point
/// sampling the BSDF at the intersection.
pub fn sample_lights(wo: &Vec3<f64>, 
                     isect: &Intersection,
                     scene: &Scene) -> Spectrum {
    let bsdf = &isect.bsdf;
    let mut L = na::zero();
    for light in &scene.lights {
        let (li, wi) = light.sample(&isect.point);
        if li != na::zero() && !light.shadow(&isect.point, scene) {
            let dot: f64 = na::dot(&isect.normal, &wi);
            if dot > 0.0 {
                L = L + bsdf.f(wo, &wi, BSDF_ALL) * li * dot;
            }
        }
    }
    L
}

/// Find the specular reflection component at a surface point.
pub fn specular_reflect(ray: &Ray,
                        isect: &Intersection, 
                        scene: &Scene,
                        renderer: &Renderer) -> Spectrum {
    let wo = -(*ray.dir());
    let bsdf = &isect.bsdf;
    let (_, wi, _) = bsdf.sample_f(&wo, BSDF_REFLECTION);
    if wi != na::zero() {
        let ray = Ray::new_with_depth(isect.point, -wi, ray.depth + 1);
        renderer.render(&ray, scene)
    } else {
        na::zero()
    }
}

/// Find the specular transmission component at a surface point.
pub fn specular_transmission(ray: &Ray,
                             isect: &Intersection, 
                             scene: &Scene,
                             renderer: &Renderer) -> Spectrum {
    let wo = -(*ray.dir());
    let bsdf = &isect.bsdf;
    let(_, wi, _) = bsdf.sample_f(&wo, BSDF_TRANSMISSION);
    na::zero()
}

pub trait Renderer {
	fn render(&self, ray: &Ray, scene: &Scene) -> Spectrum;
}

pub struct StandardRenderer {
    depth: i32
}

impl StandardRenderer {
    pub fn new(depth: i32) -> StandardRenderer {
        StandardRenderer {
            depth : depth
        }
    }
}

impl Renderer for StandardRenderer {
    fn render(&self, ray: &Ray, scene: &Scene) -> Spectrum {
        let isect_opt = scene.trace(ray);

        match isect_opt {
            Some(isect) => {
                let wo = -(*ray.dir());
                let mut L = sample_lights(&wo, &isect, scene);

                if ray.depth < self.depth {
                    L = L + specular_reflect(ray, &isect, scene, self);
                    L = L + specular_transmission(ray, &isect, scene, self);
                }
                L
            },
            None => na::zero()
        }
    }
}