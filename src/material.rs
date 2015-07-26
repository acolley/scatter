
use std::ops::{Deref};

use na::{Vec3};

use surface::{SurfaceIntegrator};
use texture::{Texture};

pub trait Material {
    fn get_surface<'a>(&'a self) -> &'a SurfaceIntegrator;
    fn get_texture<'a>(&'a self) -> &'a Texture;
}

pub struct StandardMaterial {
    pub surface: Box<SurfaceIntegrator>,
    pub texture: Box<Texture>
}

impl StandardMaterial {
    pub fn new<T: 'static + Texture, S: 'static + SurfaceIntegrator>(
        surface: Box<S>,
        texture: Box<T>) -> StandardMaterial {
        StandardMaterial {
            surface : surface as Box<SurfaceIntegrator>,
            texture : texture as Box<Texture>
        }
    }
}

impl Material for StandardMaterial {
    fn get_surface<'a>(&'a self) -> &'a SurfaceIntegrator {
        self.surface.deref()
    }

    fn get_texture<'a>(&'a self) -> &'a Texture {
        self.texture.deref()
    }
}