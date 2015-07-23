
use na;
use na::{ApproxEq, Vec3};

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

#[test]
fn test_unit_y() {
    let vy = Vec3::y();
    let (vz, vx) = coordinate_system(&vy);
    assert_approx_eq!(vx, -Vec3::x());
    assert_approx_eq!(vz, -Vec3::z());
}