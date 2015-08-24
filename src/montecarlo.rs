
use std::f64::consts;

use na::{Vec3};

#[inline]
pub fn concentric_sample_disc(u1: f64, u2: f64) -> (f64, f64) {
    // remap into [-1, 1]
    let sx = 2.0 * u1 - 1.0;
    let sy = 2.0 * u2 - 1.0;

    // map the square to (r, theta)

    // handle degeneracy at the origin
    if sx == 0.0 && sy == 0.0 {
        return (0.0, 0.0);
    }

    let (r, theta) = if sx >= -sy {
        if sx > sy {
            // first region of disc
            if sy > 0.0 {
                (sx, sy/sx)
            } else {
                (sx, 8.0 + sy/sx)
            }
        } else {
            // second region
            (sy, 2.0 - sx/sy)
        }
    } else {
        if sx <= sy {
            // third region of disc
            (-sx, 4.0 - sy/-sx)
        } else {
            // fourth region of disc
            (-sy, 6.0 + sx/-sy)
        }
    };

    let theta = theta * (consts::FRAC_PI_4);
    let dx = r * theta.cos();
    let dy = r * theta.sin();
    (dx, dy)
}

#[inline]
pub fn cosine_sample_hemisphere(u1: f64, u2: f64) -> Vec3<f64> {
	let (x, y) = concentric_sample_disc(u1, u2);
    let z = f64::max(0.0, 1.0 - x*x - y*y).sqrt();
    Vec3::new(x, y, z)
}
