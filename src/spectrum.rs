use na;
use na::{Vec3};

pub type Spectrum = Vec3<f64>;

// pub struct Spectrum(Vec3<f64>);

// impl Spectrum {
//     pub fn black() -> Spectrum {
//         Spectrum(na::zero())
//     }

//     pub fn white() -> Spectrum {
//         Spectrum(Vec3::new(1.0, 1.0, 1.0))
//     }

//     pub fn red() -> Spectrum {
//         Spectrum(Vec3::new(1.0, 0.0, 0.0))
//     }

//     pub fn green() -> Spectrum {
//         Spectrum(Vec3::new(0.0, 1.0, 0.0))
//     }

//     pub fn blue() -> Spectrum {
//         Spectrum(Vec3::new(0.0, 0.0, 1.0))
//     }

//     pub fn is_black(&self) -> bool {
//         let Spectrum(v) = *self;
//         v == na::zero()
//     }
// }