
use math;
use spectrum::{Spectrum};

use na;
use na::{Mat3, Norm, Pnt3, Rot3, Transform, Vec3};
use ncollide::ray::{Ray3};

use scene::{Scene};

bitflags! {
    flags BxDFType: u32 {
        const BSDF_REFLECTION       = 0b00000001,
        const BSDF_TRANSMISSION     = 0b00000010,
        const BSDF_DIFFUSE          = 0b00000100,
        const BSDF_GLOSSY           = 0b00001000,
        const BSDF_SPECULAR         = 0b00010000,
        const BSDF_ALL_TYPES        = BSDF_DIFFUSE.bits
                                      | BSDF_GLOSSY.bits
                                      | BSDF_SPECULAR.bits,
        const BSDF_ALL_REFLECTION   = BSDF_REFLECTION.bits
                                      | BSDF_ALL_TYPES.bits,
        const BSDF_ALL_TRANSMISSION = BSDF_TRANSMISSION.bits
                                      | BSDF_ALL_TYPES.bits,
        const BSDF_ALL              = BSDF_ALL_REFLECTION.bits
                                      | BSDF_ALL_TRANSMISSION.bits
    }
}

pub trait BxDF {
    fn pdf(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> f64;
    /// Returns wi and the pdf
    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64);
    fn f(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> Spectrum;
    fn bxdf_type(&self) -> BxDFType;

    fn matches_flags(&self, bxdf_type: BxDFType) -> bool {
        !(self.bxdf_type() & bxdf_type).is_empty()
    }
}

pub struct Diffuse;

impl BxDF for Diffuse {
    fn pdf(&self, _: &Vec3<f64>, _: &Vec3<f64>) -> f64 { 0.0 }

    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64) {
        (na::zero(), na::zero(), self.pdf(wo, &na::zero()))
    }

    fn f(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> Spectrum {
        na::zero()
    }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_DIFFUSE
    }
}

/// A structure representing a Bidirectional Reflectance
/// Distribution Function for Specular Reflection.
/// This models the amount of incident
/// light reflected from a surface and in what direction(s).
pub struct SpecularReflection;

impl BxDF for SpecularReflection {
    fn pdf(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> f64 {
        // TODO: implement this
        0.0
    }

    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64) {
        let n = Vec3::z();
        let mut wi = *wo - n * 2.0 * (na::dot(wo, &n));
        wi.normalize_mut();
        (na::zero(), wi, self.pdf(wo, &wi))
    }

    fn f(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> Spectrum {
        na::zero()
    }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_REFLECTION
    }
}

/// A structure representing a Bidirectional Transmission
/// Distribution Function for Specular Transmission. 
/// This models the amount of incident
/// light transmitted through the surface and in what direction(s).
pub struct SpecularTransmission;

impl BxDF for SpecularTransmission {
    fn pdf(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> f64 {
        // TODO: implement this
        0.0
    }

    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64) {
        // TODO: implement this
        (na::zero(), na::zero(), self.pdf(wo, &na::zero()))
    }

    /// wi: Incident light direction in local space
    /// wo: Outgoing light direction in local space
    fn f(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> Spectrum {
        na::zero()
    }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_TRANSMISSION
    }
}

pub struct BSDF {
    normal: Vec3<f64>,
    world_to_local: Rot3<f64>,
    bxdfs: Vec<Box<BxDF>>
}

impl BSDF {
    pub fn new(normal: Vec3<f64>) -> BSDF {
        BSDF {
            normal : normal,
            world_to_local : BSDF::world_to_local_from_normal(&normal),
            bxdfs : Vec::new()
        }
    }

    pub fn new_with_bxdfs(normal: Vec3<f64>, bxdfs: Vec<Box<BxDF>>) -> BSDF {
        BSDF {
            normal : normal,
            world_to_local : BSDF::world_to_local_from_normal(&normal),
            bxdfs : bxdfs
        }
    }

    fn world_to_local_from_normal(normal: &Vec3<f64>) -> Rot3<f64> {
        let (tangent, binormal) = math::coordinate_system(&normal);
        unsafe {
            Rot3::new_with_mat(Mat3::new(
                tangent.x, tangent.y, tangent.z,
                binormal.x, binormal.y, binormal.z,
                normal.x, normal.y, normal.z
            ))
        }
    }

    #[inline]
    pub fn add_bxdf<T: 'static + BxDF>(&mut self, x: Box<T>) {
        self.bxdfs.push(x as Box<BxDF>);
    }

    #[inline]
    pub fn world_to_local(&self, v: &Vec3<f64>) -> Vec3<f64> {
        self.world_to_local.transform(v)
    }

    #[inline]
    pub fn local_to_world(&self, v: &Vec3<f64>) -> Vec3<f64> {
        self.world_to_local.inv_transform(v)
    }

    pub fn sample_f(&self, wo_world: &Vec3<f64>, flags: BxDFType) -> (Spectrum, Vec3<f64>, f64) {
        let wo = self.world_to_local(wo_world);

        let mut bxdfs = self.bxdfs.iter().filter(|x| x.matches_flags(flags));
        // Choose the first BxDF that matches the flags given
        let (colour, wi, pdf) = match bxdfs.next() {
            Some(bxdf) => {
                bxdf.sample_f(&wo)
            },
            None => (na::zero(), na::zero(), 0.0)
        };

        let wi = self.local_to_world(&wi);

        (colour, wi, pdf)
    }

    pub fn f(&self, wo_world: &Vec3<f64>, wi_world: &Vec3<f64>, flags: BxDFType) -> Spectrum {
        // incident and outgoing directions in local space
        let wi = self.world_to_local(wi_world);
        let wo = self.world_to_local(wo_world);

        let flags = {
            if na::dot(wo_world, &self.normal) * na::dot(wi_world, &self.normal) > 0.0 {
                // ignore BTDFs as the incident ray is on the outside of the surface
                flags & !BSDF_TRANSMISSION
            } else {
                // ignore BRDFs as the incident ray is on the inside of the surface
                flags & !BSDF_REFLECTION
            }
        };

        let bxdfs = self.bxdfs.iter().filter(|x| x.matches_flags(flags));
        let mut f: Vec3<f64> = na::zero();
        for bxdf in bxdfs {
            f = f + bxdf.f(&wi, &wo);
        }
        f
    }
}