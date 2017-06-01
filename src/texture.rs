
use std::sync::Arc;

use na;
use na::Point2;

use image::{GenericImage, RgbImage};
use math::Scalar;
use spectrum::Spectrum;

pub trait Texture {
    fn sample(&self, uv: &Option<Point2<f64>>) -> Spectrum;
}

/// A Texture that just has a single
/// colour at any point on the surface.
pub struct ConstantTexture {
    colour: Spectrum,
}

impl ConstantTexture {
    pub fn new(colour: Spectrum) -> ConstantTexture {
        ConstantTexture { colour: colour }
    }
}

impl Texture for ConstantTexture {
    #[inline]
    fn sample(&self, _: &Option<Point2<f64>>) -> Spectrum {
        self.colour
    }
}

pub struct ImageTexture {
    data: Arc<RgbImage>,
}

impl ImageTexture {
    // TODO: make it take an Rc<RgbImage> so that the image
    // can be shared instead of copied
    pub fn new(data: Arc<RgbImage>) -> ImageTexture {
        ImageTexture { data: data }
    }
}

impl Texture for ImageTexture {
    fn sample(&self, uv: &Option<Point2<f64>>) -> Spectrum {
        match *uv {
            Some(uv) => {
                let (width, height) = self.data.dimensions();
                let x = (uv.x * width as f64).round() as u32 % width;
                let y = (uv.y * height as f64).round() as u32 % height;
                let p = self.data.get_pixel(x, y);
                Spectrum::new(p[0] as Scalar / 255.0,
                              p[1] as Scalar / 255.0,
                              p[2] as Scalar / 255.0)
            }
            None => na::zero(),
        }
    }
}
