
use na;
use na::{Norm, Pnt3, Vec3};
use ncollide::ray::{Ray3};

use scene::{Scene};
use spectrum::{Spectrum};

pub trait Light {
    fn colour(&self) -> &Spectrum;

    /// Sample the light given a point and its shading
    /// normal in world space, returning a Spectrum and
    /// a normalized vector indicating the
    /// incident light direction.
    fn sample(&self, p: &Pnt3<f64>) -> (Spectrum, Vec3<f64>);

    fn shadow(&self, p: &Pnt3<f64>, scene: &Scene) -> bool;
}

pub struct PointLight {
    intensity: f64,
    colour: Spectrum,
	position: Pnt3<f64>,
    radius: f64
}

impl PointLight {
    pub fn new(intensity: f64, colour: Spectrum, position: Pnt3<f64>, radius: f64) -> PointLight {
        PointLight {
            intensity : intensity,
            colour : colour,
            position : position,
            radius : radius
        }
    }
}

impl Light for PointLight {
    fn colour(&self) -> &Spectrum { &self.colour }

    /// Give the amount of incident light at a particular
    /// point in the scene.
    fn sample(&self, p: &Pnt3<f64>) -> (Spectrum, Vec3<f64>) {
        let mut wi = self.position - *p;
        let dist = wi.sqnorm();
        wi.normalize_mut();
        if dist > 0.0 && dist <= self.radius * self.radius {
            let attenuation = (1.0 / dist) * self.radius;
            let li = self.colour * self.intensity * attenuation;
            (li, wi)
        } else {
            (na::zero(), wi)
        }
    }

    /// Is the point p in shadow cast by this light?
    fn shadow(&self, p: &Pnt3<f64>, scene: &Scene) -> bool {
        let dist = (self.position - *p).norm();
        let mut dir = self.position - *p;
        dir.normalize_mut();
        let ray = Ray3::new(*p, dir);
        scene.intersections(&ray).iter()
                                 .any(|&x| x < dist)
    }
}

pub struct DirectionalLight {
    intensity: f64,
    colour: Spectrum,
    direction: Vec3<f64>
}

impl DirectionalLight {
    pub fn new(intensity: f64, colour: Spectrum, direction: Vec3<f64>) -> DirectionalLight {
        DirectionalLight {
            intensity : intensity,
            colour : colour,
            direction : direction
        }
    }
}

impl Light for DirectionalLight {
    fn colour(&self) -> &Spectrum { &self.colour }

    fn sample(&self, _: &Pnt3<f64>) -> (Spectrum, Vec3<f64>) {
        (self.colour * self.intensity, -self.direction)
    }

    fn shadow(&self, _: &Pnt3<f64>, _: &Scene) -> bool {
        false
    }
}

// #[test]
// fn test_DirectionalLight_sample() {
//     // point is irrelevant for a directional light
//     let l = DirectionalLight::new(1.0, na::one(), Vec3::y());
//     let p = Pnt3::new(0.0, 0.0, 0.0);
//     let n = -Vec3::y();
//     let value = l.sample(&p, &n);
//     assert_approx_eq!(value, na::one());
// }

// #[test]
// fn test_PointLight_sample() {
//     let l = PointLight::new(1.0, na::one(), Pnt3::new(0.0, 0.0, 0.0), 1.0);
//     let p = Pnt3::new(0.0, 0.0, 0.0);
//     let n = Vec3::x();
//     let value = l.sample(&p, &n);
//     assert_approx_eq!(value, na::one());
// }