
use na;
use na::Point2;

use bxdf::{BSDF, Lambertian, FresnelConductor, FresnelDielectric, SpecularReflection,
           SpecularTransmission};
use math::Normal;
use spectrum::Spectrum;
use texture::Texture;

pub trait Material {
    fn get_bsdf(&self, normal: &Normal, uvs: &Option<Point2<f64>>) -> BSDF;
}

pub struct DiffuseMaterial {
    pub texture: Box<Texture + Sync + Send>,
}

impl DiffuseMaterial {
    pub fn new(texture: Box<Texture + Sync + Send>) -> DiffuseMaterial {
        DiffuseMaterial { texture: texture }
    }
}

impl Material for DiffuseMaterial {
    fn get_bsdf(&self, normal: &Normal, uvs: &Option<Point2<f64>>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        let f = self.texture.sample(uvs);
        bsdf.add_bxdf(Box::new(Lambertian::new(f)));
        bsdf
    }
}

pub struct GlassMaterial;

impl Material for GlassMaterial {
    fn get_bsdf(&self, normal: &Normal, _: &Option<Point2<f64>>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        // refractive index for glass is 1.5
        bsdf.add_bxdf(Box::new(SpecularTransmission::new(Spectrum::new(1.0, 1.0, 1.0), 1.0, 1.5)));
        bsdf.add_bxdf(Box::new(SpecularReflection::new(Spectrum::new(1.0, 1.0, 1.0),
                                                       Box::new(FresnelDielectric::new(1.0,
                                                                                       1.5)))));
        bsdf
    }
}

pub struct MirrorMaterial;

impl Material for MirrorMaterial {
    fn get_bsdf(&self, normal: &Normal, _: &Option<Point2<f64>>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        bsdf.add_bxdf(Box::new(
            SpecularReflection::new(
                Spectrum::new(1.0, 1.0, 1.0),
                Box::new(FresnelConductor::new(na::zero(),
                                               Spectrum::new(1.0, 1.0, 1.0))))));
        bsdf
    }
}
