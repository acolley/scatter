use na;
use na::Vector3;

pub type Spectrum = Vector3<f64>;

// pub struct Spectrum(Vector3<f64>);

// impl Spectrum {
//     pub fn black() -> Spectrum {
//         Spectrum(na::zero())
//     }

//     pub fn white() -> Spectrum {
//         Spectrum(Vector3::new(1.0, 1.0, 1.0))
//     }

//     pub fn red() -> Spectrum {
//         Spectrum(Vector3::new(1.0, 0.0, 0.0))
//     }

//     pub fn green() -> Spectrum {
//         Spectrum(Vector3::new(0.0, 1.0, 0.0))
//     }

//     pub fn blue() -> Spectrum {
//         Spectrum(Vector3::new(0.0, 0.0, 1.0))
//     }

//     pub fn is_black(&self) -> bool {
//         let Spectrum(v) = *self;
//         v == na::zero()
//     }
// }
