
use na;
use na::{Pnt3, Vec3};
use ncollide::ray::{LocalRayCast, Ray};

use light::{Light, LightType};
use spectrum::{Spectrum};
use surface::{Diffuse, SurfaceIntegrator};

pub struct Scene {
    spheres: Vec<Box<LocalRayCast<Pnt3<f64>>>>,
	lights: Vec<Box<Light>>
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            spheres: Vec::new(),
            lights: Vec::new()
        }
    }

    pub fn add_sphere(&mut self, sphere: Box<LocalRayCast<Pnt3<f64>>>) {
        self.spheres.push(sphere);
    }

    pub fn add_light(&mut self, light: Box<Light>) {
        self.lights.push(light);
    }

    pub fn trace(&mut self, ray: &Ray<Pnt3<f64>>, depth: isize) -> Spectrum {
        let surface = Diffuse;
        let mut colour: Spectrum = na::zero();
        for sphere in self.spheres.iter_mut() {
            match sphere.toi_and_normal_with_ray(ray, true) {
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