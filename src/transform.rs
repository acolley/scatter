extern crate nalgebra as na;

use self::na::{ApproxEq, BaseFloat, Quat, Rot3, Rotation, UnitQuat, Vec3};

pub struct Transform {
	position: Vec3<f64>,
	rotation: Rot3<f64>,
	scale: f64 // TODO: make this a Vec3?
}

impl Transform {
    pub fn identity() -> Transform {
        Transform {
            position : na::zero(),
            rotation : na::one(),
            scale : 1f64
        }
    }

    pub fn new(position: Vec3<f64>, rotation: Rot3<f64>, scale: f64) -> Transform {
        Transform {
            position : position,
            rotation : rotation,
            scale : scale
        }
    }

    /// Translate the Transform by delta and return a
    /// new Transform with the translation applied
    pub fn translate(&self, delta: &Vec3<f64>) -> Transform {
        Transform::new(
            self.position + *delta,
            self.rotation,
            self.scale)
    }

    /// Translate the Transform by delta in place
    pub fn translate_mut(&mut self, delta: &Vec3<f64>) {
        self.position = self.position + *delta;
    }

    /// Rotate the Transform by delta and return a
    /// new Transform with the rotation applied
    pub fn rotate(&self, delta: &Vec3<f64>) -> Transform {
        Transform::new(
            self.position,
            self.rotation.append_rotation(delta),
            self.scale)
    }

    /// Rotate the Transform by delta in place
    pub fn rotate_mut(&mut self, delta: &Vec3<f64>) {
        self.rotation.append_rotation_mut(delta);
    }

    /// Scale the Transform by delta and return a
    /// new Transform with the scaling applied
    pub fn scale(&self, scale: f64) -> Transform {
        Transform::new(
            self.position,
            self.rotation,
            scale)
    }

    /// Scale the Transform by delta in place
    pub fn scale_mut(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// Apply the Transform to a vector
    pub fn transform(&self, vec: &Vec3<f64>) -> Vec3<f64> {
        let scaled = *vec * self.scale;
        na::rotate(&self.rotation, &scaled) + self.position
    }
}

#[test]
fn test_transform_scale() {
    let transform = Transform::new(
        na::zero(), 
        na::one(), 
        2f64);
    let vec = transform.transform(&Vec3::new(1f64, 1.0, 1.0));
    assert_approx_eq!(&vec, &Vec3::new(2f64, 2.0, 2.0));
}

#[test]
fn test_transform_translate() {
    let transform = Transform::new(
        Vec3::new(10f64, 0.0, 0.0), 
        na::one(), 
        na::one());
    let vec = transform.transform(&Vec3::new(0f64, 0.0, 0.0));
    assert_approx_eq!(&vec, &Vec3::new(10f64, 0.0, 0.0));
}

#[test]
fn test_transform_rotate() {
    let rotation = Rot3::new(Vec3::new(0f64, 0.0, 0.5 * <f64 as BaseFloat>::pi()));
    let transform = Transform::new(na::zero(), rotation, 1f64);
    let vec = transform.transform(&Vec3::new(1f64, 0.0, 0.0));
    assert_approx_eq!(&vec, &Vec3::new(0.0, 1.0, 0.0));
}

#[test]
fn test_transform_scale_translate_rotate() {
    let scale = 3f64;
    let rotation = Rot3::new(Vec3::new(0f64, 0.0, 0.5 * <f64 as BaseFloat>::pi()));
    let translation = Vec3::new(0f64, 2.0, 1.0);
    let transform = Transform::new(translation, rotation, scale);
    let vec = transform.transform(&Vec3::new(1f64, 0.0, 0.0));
    assert_approx_eq!(&vec, &Vec3::new(0f64, 5.0, 1.0));
}

#[test]
fn test_translate() {
    let transform = Transform::new(na::zero(), na::one(), 1f64);
    assert_approx_eq!(&transform.position, &Vec3::new(0f64, 0.0, 0.0));
    let translation = Vec3::new(0f64, 2.0, 0.0);
    let transform = transform.translate(&translation);
    assert_approx_eq!(&transform.position, &Vec3::new(0f64, 2.0, 0.0));
}

#[test]
fn test_translate_mut() {
    let mut transform = Transform::new(na::zero(), na::one(), 1f64);
    assert_approx_eq!(&transform.position, &Vec3::new(0f64, 0.0, 0.0));
    let translation = Vec3::new(0f64, 2.0, 0.0);
    transform.translate_mut(&translation);
    assert_approx_eq!(&transform.position, &Vec3::new(0f64, 2.0, 0.0));
}

#[test]
fn test_scale() {
    let transform = Transform::new(na::zero(), na::one(), 1f64);
    assert_approx_eq!(&transform.scale, &1f64);
    let scale = 10f64;
    let transform = transform.scale(scale);
    assert_approx_eq!(&transform.scale, &10f64);
}

#[test]
fn test_scale_mut() {
    let mut transform = Transform::new(na::zero(), na::one(), 1f64);
    assert_approx_eq!(&transform.scale, &1f64);
    let scale = 10f64;
    transform.scale_mut(scale);
    assert_approx_eq!(&transform.scale, &10f64);
}

#[test]
fn test_rotate() {

}

#[test]
fn test_rotate_mut() {

}