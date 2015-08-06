
use na;
use na::{Norm, Vec3};

pub type Normal = Vec3<f64>;

pub fn coordinate_system(v1: &Vec3<f64>) -> (Vec3<f64>, Vec3<f64>) {
    let v2 = {
        if v1.x.abs() > v1.y.abs() {
            let invlen = 1.0 / (v1.x * v1.x + v1.z * v1.z).sqrt();
            Vec3::new(-v1.z * invlen, 0.0, v1.x * invlen)
        } else {
            let invlen = 1.0 / (v1.y * v1.y + v1.z * v1.z).sqrt();
            Vec3::new(0.0, v1.z * invlen, -v1.y * invlen)
        }
    };
    let v3 = na::cross(v1, &v2);
    (v2, v3)
}

/// Reflect a vector `v` around an arbitrary normal vector
/// `n`. The normal is assumed to be normalized.
pub fn reflect(v: &Vec3<f64>, n: &Normal) -> Vec3<f64> {
    let mut reflected = *v - *n * 2.0 * (na::dot(v, n));
    reflected.normalize_mut();
    reflected
}

pub trait Clamp {
    fn clamp(&self, min: Self, max: Self) -> Self;
}

impl Clamp for f64 {
    fn clamp(&self, min: f64, max: f64) -> f64 {
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