
use na;
use na::{Norm, Pnt3, Vec3};

use math::{Normal, Point, Scalar, Vector};
use ray::{Ray};
use scene::{Scene};
use spectrum::{Spectrum};

pub trait Light {
    fn colour(&self) -> &Spectrum;

    fn is_delta(&self) -> bool;

    /// Sample the light given a point and its shading
    /// normal in world space, returning a Spectrum and
    /// a normalized vector indicating the
    /// incident light direction.
    fn sample(&self, p: &Point) -> (Spectrum, Vector);

    #[inline]
    fn emitted(&self, wi: &Vector) -> Spectrum { na::zero() }

    fn shadow(&self, p: &Point, scene: &Scene) -> bool;
}

pub struct PointLight {
    intensity: Scalar,
    colour: Spectrum,
	position: Point,
    radius: Scalar
}

impl PointLight {
    pub fn new(intensity: Scalar, colour: Spectrum, position: Point, radius: Scalar) -> PointLight {
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

    fn is_delta(&self) -> bool { true }

    /// Give the amount of incident light at a particular
    /// point in the scene.
    fn sample(&self, p: &Point) -> (Spectrum, Vector) {
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
    fn shadow(&self, p: &Point, scene: &Scene) -> bool {
        let dist = na::dist(&self.position, p);
        let mut dir = self.position - *p;
        dir.normalize_mut();
        let ray = Ray::new(*p, dir);
        scene.intersections(&ray).iter()
                                 .any(|&x| x < dist)
    }
}

pub struct DirectionalLight {
    intensity: Scalar,
    colour: Spectrum,
    direction: Vector
}

impl DirectionalLight {
    pub fn new(intensity: Scalar, colour: Spectrum, direction: Vector) -> DirectionalLight {
        DirectionalLight {
            intensity : intensity,
            colour : colour,
            direction : direction
        }
    }
}

impl Light for DirectionalLight {
    #[inline]
    fn colour(&self) -> &Spectrum { &self.colour }

    fn is_delta(&self) -> bool { true }

    #[inline]
    fn sample(&self, _: &Point) -> (Spectrum, Vector) {
        (self.colour * self.intensity, -self.direction)
    }

    #[inline]
    fn shadow(&self, _: &Point, _: &Scene) -> bool {
        // No point can be in shadow from a global directional light
        false
    }
}

pub trait AreaLight : Light {
    fn is_delta(&self) -> bool { false }

    fn radiance(&self, p: &Point, n: &Normal, w: &Vector) -> Spectrum;
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