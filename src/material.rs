
use na;
use na::{Pnt2, Vec3};

use bxdf::{BSDF, Lambertian, FresnelConductor, FresnelDielectric, SpecularReflection, SpecularTransmission};
use texture::{Texture};

pub trait Material {
    fn get_bsdf(&self, normal: &Vec3<f64>, uvs: &Option<Pnt2<f64>>) -> BSDF;
}

pub struct DiffuseMaterial {
    pub texture: Box<Texture + Sync + Send>
}

impl DiffuseMaterial {
    pub fn new<T: 'static + Texture + Sync + Send>(
        texture: Box<T>) -> DiffuseMaterial {
        DiffuseMaterial {
            texture : texture as Box<Texture + Sync + Send>
        }
    }
}

impl Material for DiffuseMaterial {
    fn get_bsdf(&self, normal: &Vec3<f64>, uvs: &Option<Pnt2<f64>>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        let f = self.texture.sample(uvs);
        bsdf.add_bxdf(Box::new(Lambertian::new(f)));
        bsdf
    }
}

pub struct GlassMaterial;

impl Material for GlassMaterial {
    fn get_bsdf(&self, normal: &Vec3<f64>, _: &Option<Pnt2<f64>>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        // refractive index for glass is 1.5
        bsdf.add_bxdf(Box::new(
            SpecularTransmission::new(
                Vec3::new(1.0, 1.0, 1.0), 
                1.0,
                1.5)));
        bsdf.add_bxdf(Box::new(
            SpecularReflection::new(
                Vec3::new(1.0, 1.0, 1.0),
                Box::new(FresnelDielectric::new(1.0, 1.5)))));
        bsdf
    }
}

pub struct MirrorMaterial;

impl Material for MirrorMaterial {
    fn get_bsdf(&self, normal: &Vec3<f64>, _: &Option<Pnt2<f64>>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        bsdf.add_bxdf(Box::new(
            SpecularReflection::new(
                Vec3::new(1.0, 1.0, 1.0), 
                Box::new(FresnelConductor::new(na::zero(),
                                               Vec3::new(1.0, 1.0, 1.0))))));
        bsdf
    }
}