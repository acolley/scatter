
use math;
use spectrum::{Spectrum};

use na;
use na::{Mat3, Norm, Pnt3, Rot3, Transform, Vec3};
use ncollide::ray::{Ray3};

use math::{Clamp, reflect};
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

/// Return Cos Theta for a normalized vector
/// in normal space.
fn cos_theta(v: &Vec3<f64>) -> f64 {
    v.z
}

fn sin_theta2(v: &Vec3<f64>) -> f64 {
    f64::max(0.0, 1.0 - cos_theta(v)*cos_theta(v))
}

fn sin_theta(v: &Vec3<f64>) -> f64 {
    sin_theta2(v).sqrt()
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

pub struct Diffuse {
    colour: Spectrum
}

impl Diffuse {
    pub fn new(colour: Spectrum) -> Diffuse {
        Diffuse {
            colour : colour
        }
    }
}

impl BxDF for Diffuse {
    fn pdf(&self, _: &Vec3<f64>, _: &Vec3<f64>) -> f64 { 0.0 }

    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64) {
        (na::zero(), na::zero(), self.pdf(wo, &na::zero()))
    }

    /// diffuse surfaces emit the same amount of light in all directions
    fn f(&self, _: &Vec3<f64>, _: &Vec3<f64>) -> Spectrum {
        self.colour
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
pub struct SpecularReflection {
    R: Spectrum,
    // we store a trait object here as a reflective surface
    // can be something that has reflection and/or transmission
    // (e.g. metal or frosted glass)
    fresnel: Box<Fresnel>
}

impl SpecularReflection {
    pub fn new<F: 'static + Fresnel>(R: Spectrum, fresnel: Box<F>) -> SpecularReflection {
        SpecularReflection {
            R : R,
            fresnel : fresnel as Box<Fresnel>
        }
    }
}

impl BxDF for SpecularReflection {
    /// The Probability Distribution Function for use in
    /// Monte Carlo sampling.
    fn pdf(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> f64 { 0.0 }

    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64) {
        let wi = Vec3::new(-wo.x, -wo.y, wo.z);
        let L = self.fresnel.evaluate(cos_theta(wo)) * self.R / cos_theta(&wi).abs();
        (L, wi, 1.0)
    }

    /// Specular reflection only produces light in a single direction
    /// given by the sample_f method
    fn f(&self, _: &Vec3<f64>, _: &Vec3<f64>) -> Spectrum { na::zero() }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_REFLECTION | BSDF_SPECULAR
    }
}

/// A structure representing a Bidirectional Transmission
/// Distribution Function for Specular Transmission. 
/// This models the amount of incident
/// light transmitted through the surface and in what direction(s).
pub struct SpecularTransmission {
    T: Spectrum,
    etai: f64,
    etat: f64,
    fresnel: FresnelDielectric
}

impl SpecularTransmission {
    pub fn new(T: Spectrum, etai: f64, etat: f64) -> SpecularTransmission {
        SpecularTransmission {
            T : T,
            etai : etai,
            etat : etat,
            fresnel : FresnelDielectric::new(etai, etat)
        }
    }
}

impl BxDF for SpecularTransmission {
    fn pdf(&self, wo: &Vec3<f64>, wi: &Vec3<f64>) -> f64 { 0.0 }

    fn sample_f(&self, wo: &Vec3<f64>) -> (Spectrum, Vec3<f64>, f64) {
        let entering = cos_theta(wo) > 0.0;
        let (etai, etat) = if entering {
            (self.etai, self.etat)
        } else {
            (self.etat, self.etai)
        };

        // calculate transmitted ray direction
        let sini2 = sin_theta2(wo);
        let eta = etai / etat;
        let sint2 = eta * eta * sini2;

        // total internal reflection
        if sint2 > 1.0 {
            return (na::zero(), na::zero(), 0.0);
        }

        let cost = if entering {
            -f64::max(0.0, 1.0 - sint2)
        } else {
            f64::max(0.0, 1.0 - sint2)
        };

        let sint_over_sini = eta;
        let wi = Vec3::new(sint_over_sini * -wo.x, sint_over_sini * -wo.y, cost);
        let F = self.fresnel.evaluate(cos_theta(wo));
        let transmitted = (Vec3::new(1.0, 1.0, 1.0) - F) * self.T / cos_theta(&wi).abs();
        (transmitted, wi, 1.0)
    }

    /// Specular transmission only produces light in a single direction
    /// given by the sample_f method.
    fn f(&self, _: &Vec3<f64>, _: &Vec3<f64>) -> Spectrum { na::zero() }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_TRANSMISSION | BSDF_SPECULAR
    }
}

pub struct BSDF {
    normal: Vec3<f64>,
    world_to_local: Rot3<f64>,
    bxdfs: Vec<Box<BxDF>>
}

impl BSDF {
    pub fn new(normal: Vec3<f64>) -> BSDF {
        Self::new_with_bxdfs(normal, Vec::new())
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

/// Return the amount of energy reflected from a dielectric
/// surface (i.e. a non-conductor like glass).
fn fr_diel(cosi: f64, cost: f64, etai: &Spectrum, etat: &Spectrum) -> Spectrum {
    let rparl = ((*etat * cosi) - (*etai * cost)) /
                ((*etat * cosi) + (*etai * cost));
    let rperp = ((*etai * cosi) - (*etat * cost)) /
                ((*etai * cosi) + (*etat * cost));
    (rparl * rparl + rperp * rperp) / 2.0
}

/// Return the amount of energy reflected from a conductor.
fn fr_cond(cosi: f64, eta: &Spectrum, k: &Spectrum) -> Spectrum {
    let tmp = (*eta * *eta + *k * *k) * cosi * cosi;
    let rparl2 = (tmp - (*eta * 2.0 * cosi) + 1.0) /
                 (tmp + (*eta * 2.0 * cosi) + 1.0);
    let tmp_f = *eta * *eta + *k * *k;
    let rperp2 = (tmp_f - (*eta * 2.0 * cosi) + cosi * cosi) /
                 (tmp_f + (*eta * 2.0 * cosi) + cosi * cosi);
    (rparl2 + rperp2) / 2.0
}

pub trait Fresnel {
    fn evaluate(&self, cosi: f64) -> Spectrum;
}

pub struct FresnelConductor {
    eta: Spectrum,
    k: Spectrum
}

impl FresnelConductor {
    pub fn new(eta: Spectrum, k: Spectrum) -> FresnelConductor {
        FresnelConductor {
            eta : eta,
            k : k
        }
    }
}

impl Fresnel for FresnelConductor {
    fn evaluate(&self, cosi: f64) -> Spectrum {
        fr_cond(cosi.abs(), &self.eta, &self.k)
    }
}

pub struct FresnelDielectric {
    etai: f64,
    etat: f64
}

impl FresnelDielectric {
    pub fn new(etai: f64, etat: f64) -> FresnelDielectric {
        FresnelDielectric {
            etai : etai,
            etat : etat
        }
    }
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cosi: f64) -> Spectrum {
        let cosi = cosi.clamp(-1.0, 1.0);

        // compute indices of refraction
        let entering = cosi > 0.0;
        let (etai, etat) = {
            if entering {
                (self.etai, self.etat)
            } else {
                (self.etat, self.etai)
            }
        };

        // compute sint using Snell's law
        let sint = etai / etat * f64::max(0.0, 1.0 - cosi*cosi).sqrt();
        if sint > 1.0 {
            // total internal reflection
            Vec3::new(1.0, 1.0, 1.0)
        } else {
            let cost = f64::max(0.0, 1.0 - sint*sint).sqrt();
            fr_diel(cosi.abs(), cost, &Vec3::new(etai, etai, etai), &Vec3::new(etat, etat, etat))
        }
    }
}