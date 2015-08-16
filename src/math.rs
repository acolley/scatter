
use na;
use na::{Norm, Pnt3, Vec3};

// TODO: allow scalar to be adjusted with a cfg build flag
pub type Scalar = f64;
pub type Point = Pnt3<Scalar>;
pub type Vector = Vec3<Scalar>;
pub type Normal = Vector;

pub fn coordinate_system(v1: &Vector) -> (Vector, Vector) {
    let v2 = {
        if v1.x.abs() > v1.y.abs() {
            let invlen = 1.0 / (v1.x * v1.x + v1.z * v1.z).sqrt();
            Vector::new(-v1.z * invlen, 0.0, v1.x * invlen)
        } else {
            let invlen = 1.0 / (v1.y * v1.y + v1.z * v1.z).sqrt();
            Vector::new(0.0, v1.z * invlen, -v1.y * invlen)
        }
    };
    let v3 = na::cross(v1, &v2);
    (v2, v3)
}

/// Reflect a vector `v` around an arbitrary normal vector
/// `n`. The normal is assumed to be normalized.
pub fn reflect(v: &Vector, n: &Normal) -> Vector {
    let mut reflected = *v - *n * 2.0 * (na::dot(v, n));
    reflected.normalize_mut();
    reflected
}

pub trait Clamp {
    fn clamp(&self, min: Self, max: Self) -> Self;
}

impl Clamp for Scalar {
    fn clamp(&self, min: Scalar, max: Scalar) -> Scalar {
        assert!(min <= max);
        if *self < min {
            min
        } else if *self > max {
            max
        } else {
            *self
        }
    }
}

#[test]
fn test_unit_y() {
    let vy = Vec3::y();
    let (vz, vx) = coordinate_system(&vy);
    assert_approx_eq!(vx, -Vec3::x());
    assert_approx_eq!(vz, -Vec3::z());
}

#[test]
fn test_clamp_min_f64() {
    let x = -2.0f64.clamp(-1.0, 1.0);
    assert_eq!(x, -1.0);
}

#[test]
fn test_clamp_max_f64() {
    let x = 2.0f64.clamp(-1.0, 1.0);
    assert_eq!(x, 1.0);
}