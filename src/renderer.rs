
use rand::StdRng;

use na;

use integrator::Integrator;
use ray::Ray;
use scene::Scene;
use spectrum::Spectrum;

pub trait Renderer {
    fn render(&self, ray: &Ray, scene: &Scene, rng: &mut StdRng) -> Spectrum;
}

pub struct StandardRenderer {
    integrator: Box<Integrator + Sync + Send>,
}

impl StandardRenderer {
    pub fn new(integrator: Box<Integrator + Sync + Send>) -> StandardRenderer {
        StandardRenderer { integrator: integrator }
    }
}

impl Renderer for StandardRenderer {
    fn render(&self, ray: &Ray, scene: &Scene, rng: &mut StdRng) -> Spectrum {
        let isect_opt = scene.trace(ray);

        match isect_opt {
            Some(isect) => self.integrator.integrate(ray, &isect, scene, self, rng),
            None => na::zero(),
        }
    }
}
