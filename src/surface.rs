
use na;
use na::{Pnt3, Vec3};

use light::{Light};
use spectrum::{Spectrum};

pub trait SurfaceIntegrator {
	fn sample(&self, 
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Spectrum, 
              lights: &[Box<Light>]) -> Spectrum;
}

pub struct Diffuse;

impl SurfaceIntegrator for Diffuse {
    fn sample(&self, 
              p: &Pnt3<f64>,
              n: &Vec3<f64>,
              colour: &Spectrum,
              lights: &[Box<Light>]) -> Spectrum {
        // TODO: only pass in lights that are not obscured in the direction of the point?
        let mut value = na::zero();
        for light in lights {
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
              lights: &[Box<Light>]) -> Spectrum {
        let mut value = na::zero();
        for light in lights {
            let (li, wi) = light.sample(&p);
            // calculate reflection vector
            let ri = wi - *n * 2.0 * (na::dot(&wi, n));
            
        }
        value
    }
}