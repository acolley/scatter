
use std::f64::consts;

use na;
use na::{Point3, Vector3};

use ncollide::utils::{triangle_area};
use ncollide::shape::{Ball3, Cuboid3, Triangle3, TriMesh3};

use math::{Normal, Point, Scalar, Vector, uniform_sample_sphere};
use ray::{Ray};
use scene::{Scene};
use spectrum::{Spectrum};

pub trait Light {
    fn colour(&self) -> &Spectrum;

    // fn power(&self) -> Scalar;
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
        let dist = wi.norm_squared();
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
        let dist = na::distance(&self.position, p);
        let mut dir = self.position - *p;
        dir.normalize_mut();
        let ray = Ray::new(*p, dir);
        scene.intersections(&ray).iter()
                                 .any(|&x| x < dist)
    }
}

pub struct DirectionalLight {
    colour: Spectrum,
    direction: Vector
}

impl DirectionalLight {
    pub fn new(colour: Spectrum, direction: Vector) -> DirectionalLight {
        DirectionalLight {
            colour : colour,
            direction : direction
        }
    }
}

impl Light for DirectionalLight {
    #[inline]
    fn colour(&self) -> &Spectrum { &self.colour }

    #[inline]
    fn is_delta(&self) -> bool { true }

    #[inline]
    fn sample(&self, _: &Point) -> (Spectrum, Vector) {
        (self.colour, -self.direction)
    }

    #[inline]
    fn shadow(&self, _: &Point, _: &Scene) -> bool {
        // No point can be in shadow from a global directional light
        false
    }
}

// pub struct SpotLight {
//     colour: Spectrum,
//     direction: Vector,
//     theta: Scalar
// }

// impl Light for SpotLight {
//     #[inline]
//     fn colour(&self) -> &Spectrum { &self.colour }

//     #[inline]
//     fn is_delta(&self) -> bool { true }
// }

// pub trait AreaLight : Light {
//     fn radiance(&self, p: &Point, n: &Normal, w: &Vector) -> Spectrum;
// }

// pub struct DiffuseLight {
//     emit: Spectrum,
//     area: Scalar
// }

// impl Light for DiffuseLight {
//     #[inline]
//     fn colour(&self) -> &Spectrum { &self.emit }

//     #[inline]
//     fn sample(&self, p: &Point) -> (Spectrum, Vector) {
//         (na::zero(), na::zero())
//     }
//     #[inline]
//     fn is_delta(&self) -> bool { false }

//     #[inline]
//     fn shadow(&self, p: &Point, scene: &Scene) -> bool {
//         true
//     }
// }

// impl AreaLight for DiffuseLight {
//     #[inline]
//     fn radiance(&self, p: &Point, n: &Normal, w: &Vector) -> Spectrum {
//         if na::dot(w, n) > 0.0 { self.emit } else { na::zero() }
//     }
// }

// /// A trait for designating a Shape as being an
// /// emitter for radiance.
// pub trait ShapeEmitter {
//     fn area(&self) -> Scalar;
//     fn sample(&self, u1: Scalar, u2: Scalar) -> (Point, Normal);

//     fn sample_at_point(&self, p: &Point, u1: Scalar, u2: Scalar) -> (Point, Normal) {
//         self.sample(u1, u2)
//     }
// }

// impl ShapeEmitter for Triangle3<Scalar> {
//     #[inline]
//     fn area(&self) -> Scalar {
//         triangle_area(self.a(), self.b(), self.c())
//     }

//     #[inline]
//     fn sample(&self, u1: Scalar, u2: Scalar) -> (Point, Normal) {
//         (na::zero(), na::zero())
//     }
// }

// impl ShapeEmitter for TriMesh3<Scalar> {
//     #[inline]
//     fn area(&self) -> Scalar {
//         let mut area = 0.0;
//         for idx in self.indices().iter() {
//             let p1 = self.vertices()[idx.x];
//             let p2 = self.vertices()[idx.y];
//             let p3 = self.vertices()[idx.z];
//             area = area + triangle_area(&p1, &p2, &p3);
//         }
//         area
//     }

//     #[inline]
//     fn sample(&self, u1: Scalar, u2: Scalar) -> (Point, Normal) {
//         (na::zero(), na::zero())
//     }
// }

// impl ShapeEmitter for Ball3<Scalar> {
//     #[inline]
//     fn area(&self) -> Scalar {
//         4.0 * consts::PI * self.radius() * self.radius()
//     }

//     #[inline]
//     fn sample(&self, u1: Scalar, u2: Scalar) -> (Point, Normal) {
//         let p = na::zero() + self.radius() * uniform_sample_sphere(u1, u2);
//         // TODO: need some way to transform a point into world space
//         // from the object space
//         // let n = 
//         (na::zero(), na::zero())
//     }
// }

// impl ShapeEmitter for Cuboid3<Scalar> {
//     #[inline]
//     fn area(&self) -> Scalar {
//         let he = self.half_extents();
//         2.0 * he.x * he.y * he.z
//     }

//     #[inline]
//     fn sample(&self, u1: Scalar, u2: Scalar) -> (Point, Normal) {
//         (na::zero(), na::zero())
//     }
// }

// #[test]
// fn test_DirectionalLight_sample() {
//     // point is irrelevant for a directional light
//     let l = DirectionalLight::new(1.0, na::one(), Vector3::y());
//     let p = Point3::new(0.0, 0.0, 0.0);
//     let n = -Vector3::y();
//     let value = l.sample(&p, &n);
//     assert_approx_eq!(value, na::one());
// }

// #[test]
// fn test_PointLight_sample() {
//     let l = PointLight::new(1.0, na::one(), Point3::new(0.0, 0.0, 0.0), 1.0);
//     let p = Point3::new(0.0, 0.0, 0.0);
//     let n = Vector3::x();
//     let value = l.sample(&p, &n);
//     assert_approx_eq!(value, na::one());
// }

// #[test]
// fn test_area_ball() {
//     let ball = Ball3::new(1.0);
//     let area = ball.area();
//     let expected = 4.0 * consts::PI;
//     assert_approx_eq!(area, expected);
// }

// #[test]
// fn test_area_cuboid() {
//     let cuboid = Cuboid3::new(1.0, 1.0, 1.0);
//     let area = cuboid.area();
//     let expected = 1.0;
//     assert_approx_eq!(area, expected);
// }