
use na;
use na::{Pnt3, Vec3};
use ncollide::ray::{Ray3};

use light::{Light};
use scene::{Scene};
use spectrum::{Spectrum};

pub trait SurfaceIntegrator {
	fn sample(&self, 
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Spectrum, 
              scene: &Scene,
              depth: isize) -> Spectrum;
}

pub struct Diffuse;

impl SurfaceIntegrator for Diffuse {
    fn sample(&self, 
              p: &Pnt3<f64>,
              n: &Vec3<f64>,
              colour: &Spectrum,
              scene: &Scene,
              depth: isize) -> Spectrum {
        // TODO: only pass in lights that are not obscured in the direction of the point?
        let mut value = na::zero();
        for light in &scene.lights {
            let (li, wi) = light.sample(&p);
            let dot: f64 = na::dot(n, &wi);
            if dot > 0.0 {
                let c = li * *colour * dot;
                value = value + c;
            }
        }
        value
    }
}

pub struct PerfectSpecular;

impl SurfaceIntegrator for PerfectSpecular {
    /// Simulate perfect specular reflection (i.e. a mirror)
    fn sample(&self, 
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Spectrum, 
              scene: &Scene,
              depth: isize) -> Spectrum {
        if depth <= 0 {
            return na::zero();
        }
        let mut value = na::zero();
        for light in &scene.lights {
            let (li, wi) = light.sample(&p);
            // calculate reflection vector
            let ri = wi - *n * 2.0 * (na::dot(&wi, n));
            // let reflect_ray = Ray3::new
            // scene.trace()
        }
        value
    }
}