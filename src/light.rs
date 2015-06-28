
use na;
use na::{ApproxEq, Norm, Pnt3, Vec3};

pub trait Light {
    /// Sample the light given a point and its shading
    /// normal in world space.
    fn sample(&self, p: &Pnt3<f64>, n: &Vec3<f64>) -> Vec3<f64>;
}

pub struct PointLight {
    intensity: f64,
    colour: Vec3<f64>,
	position: Pnt3<f64>,
    radius: f64
}

impl PointLight {
    pub fn new(intensity: f64, colour: Vec3<f64>, position: Pnt3<f64>, radius: f64) -> PointLight {
        PointLight {
            intensity : intensity,
            colour : colour,
            position : position,
            radius : radius
        }
    }
}

impl Light for PointLight {
    fn sample(&self, p: &Pnt3<f64>, n: &Vec3<f64>) -> Vec3<f64> {
        let mut dir = self.position - *p;
        let dist = dir.sqnorm();
        if dist > 0.0 && dist <= self.radius * self.radius {
            dir.normalize_mut();
            let dot: f64 = na::dot(n, &dir);
            if dot > 0.0 {
                // attenuation is 1 / square distance scaled
                // by the radius so that the point light has
                // higher intensity closer to the point light
                // eventually falling off to 0 at its radius
                let attenuation = (1.0 / dist) * self.radius;
                self.colour * dot * self.intensity * attenuation
            } else {
                na::zero()
            }
        } else {
            // out of range of point light so return black
            na::zero()
        }
    }
}

pub struct DirectionalLight {
    intensity: f64,
    colour: Vec3<f64>,
    direction: Vec3<f64>
}

impl DirectionalLight {
    pub fn new(intensity: f64, colour: Vec3<f64>, direction: Vec3<f64>) -> DirectionalLight {
        DirectionalLight {
            intensity : intensity,
            colour : colour,
            direction : direction
        }
    }
}

impl Light for DirectionalLight {
    fn sample(&self, p: &Pnt3<f64>, n: &Vec3<f64>) -> Vec3<f64> {
        // compute a really basic diffuse colour given a
        // point and its normal to shade based on the orientation
        // of the light relative to the normal and the colour
        // and intensity of the light
        let dot: f64 = na::dot(n, &-self.direction);
        if dot > 0.0 {
            // amount of light reaching point dependant
            // on normal of surface at the point
            self.colour * dot * self.intensity
        } else {
            // light not visible from point
            na::zero()
        }
    }
}

#[test]
fn test_DirectionalLight_sample() {
    // point is irrelevant for a directional light
    let l = DirectionalLight::new(1.0, na::one(), Vec3::y());
    let p = Pnt3::new(0.0, 0.0, 0.0);
    let n = -Vec3::y();
    let value = l.sample(&p, &n);
    assert_approx_eq!(value, na::one());
}

// #[test]
// fn test_PointLight_sample() {
//     let l = PointLight::new(1.0, na::one(), Pnt3::new(0.0, 0.0, 0.0), 1.0);
//     let p = Pnt3::new(0.0, 0.0, 0.0);
//     let n = Vec3::x();
//     let value = l.sample(&p, &n);
//     assert_approx_eq!(value, na::one());
// }