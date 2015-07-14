
use na;

use image::{GenericImage, ImageBuffer, Rgb, RgbImage};
use na::{Vec3};
use spectrum::{Spectrum};

pub trait Texture {
	fn sample(&self, u: f64, v: f64) -> Spectrum;
}

/// A Texture that just has a single
/// colour at any point on the surface.
pub struct ConstantTexture {
    colour: Spectrum
}

impl ConstantTexture {
    pub fn new(colour: Spectrum) -> ConstantTexture {
        ConstantTexture {
            colour : colour
        }
    }
}

impl Texture for ConstantTexture {
    fn sample(&self, _: f64, _: f64) -> Spectrum {
        self.colour
    }
}

pub struct ImageTexture {
    data: RgbImage
}

impl ImageTexture {
    pub fn new(data: RgbImage) -> ImageTexture {
        ImageTexture {
            data : data
        }
    }
}

impl Texture for ImageTexture {
    fn sample(&self, u: f64, v: f64) -> Spectrum {
        let (width, height) = self.data.dimensions();
        let x = (u * width as f64).round() as u32 % width;
        let y = (v * height as f64).round() as u32 % height;
        let p = self.data.get_pixel(x, y);
        Vec3::new(p[0] as f64 / 255.0, 
                  p[1] as f64 / 255.0, 
                  p[2] as f64 / 255.0)
    }
}