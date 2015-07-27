
use std::ops::{Deref};

use na::{Vec3};

use bxdf::{BSDF, Diffuse, SpecularReflection};
use texture::{Texture};

pub trait Material {
    fn get_bsdf(&self, normal: &Vec3<f64>) -> BSDF;
}

pub struct DiffuseMaterial {
    pub texture: Box<Texture>
}

impl DiffuseMaterial {
    pub fn new<T: 'static + Texture>(
        texture: Box<T>) -> DiffuseMaterial {
        DiffuseMaterial {
            texture : texture as Box<Texture>
        }
    }
}

impl Material for DiffuseMaterial {
    fn get_bsdf(&self, normal: &Vec3<f64>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        bsdf.add_bxdf(Box::new(Diffuse));
        bsdf
    }
}

pub struct SpecularMaterial;

impl Material for SpecularMaterial {
    fn get_bsdf(&self, normal: &Vec3<f64>) -> BSDF {
        let mut bsdf = BSDF::new(*normal);
        bsdf.add_bxdf(Box::new(SpecularReflection));
        bsdf
    }
}