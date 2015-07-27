
use na;
use na::{Norm, Pnt3, Vec3};

use light::{Light};
use ray::{Ray};
use scene::{Scene};
use spectrum::{Spectrum};

pub trait SurfaceIntegrator {
    /// Sample the reflected light at a particular
    /// point on a surface given the incident light
    /// direction, the point, the normal, the colour
    /// of the surface at that point and the scene
    /// for tracing more rays through the scene if
    /// required for reflected/transmitted rays.
    /// Because this is a reverse ray tracer (we trace
    /// rays from the camera first) the sample method
    /// takes the outgoing direction as a parameter.
  	fn sample(&self, 
              wo: &Vec3<f64>,
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Spectrum, 
              scene: &Scene,
              depth: u32) -> Spectrum;
}

pub struct Diffuse;

impl SurfaceIntegrator for Diffuse {
    fn sample(&self, 
              _: &Vec3<f64>,
              p: &Pnt3<f64>,
              n: &Vec3<f64>,
              colour: &Spectrum,
              scene: &Scene,
              _: u32) -> Spectrum {
        // TODO: only pass in lights that are not obscured in the direction of the point
        // in order to simulate shadows. This should also cast ray differentials?
        let mut value = na::zero();
        for light in &scene.lights {
            // cast shadow ray
            if !light.shadow(p, scene) {
                let (li, wi) = light.sample(&p);
                let dot: f64 = na::dot(n, &wi);
                if dot > 0.0 {
                    let c = li * *colour * dot;
                    value = value + c;
                }
            }
        }
        value
    }
}

pub struct Specular {
    etai: f64,
    etao: f64
}

impl Specular {
    pub fn new(etai: f64, etao: f64) -> Specular {
        Specular {
            etai : etai,
            etao : etao
        }
    }
}

impl SurfaceIntegrator for Specular {
    fn sample(&self,
              wo: &Vec3<f64>,
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Spectrum,
              scene: &Scene,
              depth: u32) -> Spectrum {
        // This should perform both reflection
        // and transmission of rays
        na::zero()
    }
}

pub struct SpecularReflection;

impl SurfaceIntegrator for SpecularReflection {
    /// Simulate perfect specular reflection (e.g. a mirror)
    fn sample(&self,
              wo: &Vec3<f64>,
              p: &Pnt3<f64>, 
              n: &Vec3<f64>, 
              colour: &Spectrum,
              scene: &Scene,
              depth: u32) -> Spectrum {
        // given that we only have directional and point lights currently
        // we do not sample those to obtain the light reflected at this point
        // on the surface, only light reflected from other surfaces is considered
        if depth <= 0 {
            return na::zero();
        }
        // TODO: attenuate amount of light energy
        // that is reflected from the surface
        let mut wi = *wo - *n * 2.0 * (na::dot(wo, n));
        wi.normalize_mut();
        let reflect_ray = Ray::new(*p, wi);
        scene.trace(&reflect_ray, depth - 1)

        // TODO: cast ray differentials for reflected rays
    }
}