
use rand::{Rng};

use na;

use integrator::{Integrator};
use ray::{Ray};
use scene::{Intersection, Scene};
use spectrum::{Spectrum};

pub trait Renderer {
	fn render<R>(&self, ray: &Ray, scene: &Scene, rng: &mut R) -> Spectrum
        where R: Rng;
}

pub struct StandardRenderer<I: Integrator + Sync + Send> {
    integrator: I
}

impl<I: Integrator + Sync + Send> StandardRenderer<I> {
    pub fn new(integrator: I) -> StandardRenderer<I> {
        StandardRenderer {
            integrator : integrator
        }
    }
}

impl<I: Integrator + Sync + Send> Renderer for StandardRenderer<I> {
    fn render<R>(&self, ray: &Ray, scene: &Scene, rng: &mut R) -> Spectrum
    where R: Rng {
        let isect_opt = scene.trace(ray);

        match isect_opt {
            Some(isect) => {
                self.integrator.integrate(ray, &isect, scene, self, rng)
            },
            None => na::zero()
        }
    }
}