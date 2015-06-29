
use na;
use na::{Pnt3, Vec3};

use light::{Light};

pub trait SurfaceIntegrator {
	fn sample(&self, p: &Pnt3<f64>, n: &Vec3<f64>, colour: &Vec3<f64>, lights: &[Box<Light>]) -> Vec3<f64>;
}

pub struct Diffuse;

impl SurfaceIntegrator for Diffuse {
    fn sample(&self, 
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Vec3<f64>, 
              lights: &[Box<Light>]) -> Vec3<f64> {
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
    fn sample(&self, p: &Pnt3<f64>, n: &Vec3<f64>, colour: &Vec3<f64>, lights: &[Box<Light>]) -> Vec3<f64> {
        na::zero()
    }
}