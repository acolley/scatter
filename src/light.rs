
use na;
use na::{ApproxEq, Norm, Pnt3, Vec3};
use ncollide::ray::{Ray, LocalRayCast};

pub enum LightType {
    Point,
    Directional
}

pub trait Light {
    fn colour(&self) -> &Vec3<f64>;
    /// Is the light visible from the point given a
    /// slice of ray castable objects that will be
    /// tested for intersection?
    fn visible_from(&self, p: &Pnt3<f64>, spheres: &[Box<LocalRayCast<Pnt3<f64>>>]) -> bool;

    /// Sample the light given a point and its shading
    /// normal in world space.
    // fn sample(&self, p: &Pnt3<f64>, n: &Vec3<f64>) -> Vec3<f64>;
    fn sample(&self, p: &Pnt3<f64>) -> (Vec3<f64>, Vec3<f64>);
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
    fn colour(&self) -> &Vec3<f64> { &self.colour }

    fn visible_from(&self, p: &Pnt3<f64>, spheres: &[Box<LocalRayCast<Pnt3<f64>>>]) -> bool {
        let mut dir = self.position - *p;
        dir.normalize_mut();
        let ray = Ray::new(*p, dir);
        for sphere in spheres {
            if sphere.intersects_ray(&ray) {
                return false;
            }
        }
        true
    }

    /// Give the amount of incident light at a particular
    /// point in the scene.
    fn sample(&self, p: &Pnt3<f64>) -> (Vec3<f64>, Vec3<f64>) {
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
    fn colour(&self) -> &Vec3<f64> { &self.colour }

    fn visible_from(&self, p: &Pnt3<f64>, spheres: &[Box<LocalRayCast<Pnt3<f64>>>]) -> bool {
        true
    }

    fn sample(&self, p: &Pnt3<f64>) -> (Vec3<f64>, Vec3<f64>) {
        (self.colour * self.intensity, -self.direction)
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