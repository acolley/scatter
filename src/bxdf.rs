
use std::f64::consts;
use math;
use spectrum::Spectrum;

use alga::linear::{ProjectiveTransformation, Transformation};
use na;
use na::{Matrix3, Rotation3, Transform};

use math::{Clamp, Scalar, Vector};
use montecarlo::cosine_sample_hemisphere;
use rand::Rng;

pub type Pdf = Scalar;

bitflags! {
    pub struct BxDFType: u32 {
        const BSDF_REFLECTION       = 0b00000001;
        const BSDF_TRANSMISSION     = 0b00000010;
        const BSDF_DIFFUSE          = 0b00000100;
        const BSDF_GLOSSY           = 0b00001000;
        const BSDF_SPECULAR         = 0b00010000;
        const BSDF_ALL_TYPES        = BSDF_DIFFUSE.bits
                                      | BSDF_GLOSSY.bits
                                      | BSDF_SPECULAR.bits;
        const BSDF_ALL_REFLECTION   = BSDF_REFLECTION.bits
                                      | BSDF_ALL_TYPES.bits;
        const BSDF_ALL_TRANSMISSION = BSDF_TRANSMISSION.bits
                                      | BSDF_ALL_TYPES.bits;
        const BSDF_ALL              = BSDF_ALL_REFLECTION.bits
                                      | BSDF_ALL_TRANSMISSION.bits;
    }
}

/// Return Cos Theta for a normalized vector
/// in normal space.
#[inline]
fn cos_theta(v: &Vector) -> Scalar {
    v.z
}

#[inline]
fn sin_theta2(v: &Vector) -> Scalar {
    Scalar::max(0.0, 1.0 - cos_theta(v) * cos_theta(v))
}

#[inline]
fn sin_theta(v: &Vector) -> Scalar {
    sin_theta2(v).sqrt()
}

#[inline]
fn same_hemisphere(w: &Vector, wp: &Vector) -> bool {
    w.z * wp.z > 0.0
}

pub trait BxDF {
    fn pdf(&self, wo: &Vector, wi: &Vector) -> Pdf {
        if same_hemisphere(wo, wi) {
            cos_theta(wi).abs() * consts::FRAC_1_PI
        } else {
            0.0
        }
    }

    /// Returns wi and the pdf
    /// Default implementation returns a hemisphere
    /// sampled direction and Pdf
    fn sample_f(&self, wo: &Vector, u1: Scalar, u2: Scalar) -> (Spectrum, Vector, Pdf) {
        // (na::zero(), na::zero(), 0.0)
        // Cosine-sample the hemisphere, flipping the direction if necessary
        let mut wi = cosine_sample_hemisphere(u1, u2);
        if wo.z < 0.0 {
            wi.z *= -1.0;
        }
        let l = self.f(wo, &wi);
        let pdf = self.pdf(wo, &wi);
        (l, wi, pdf)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum;
    fn bxdf_type(&self) -> BxDFType;

    fn matches_flags(&self, bxdf_type: BxDFType) -> bool {
        (self.bxdf_type() & bxdf_type) == self.bxdf_type()
    }
}

pub struct Lambertian {
    colour: Spectrum,
}

impl Lambertian {
    pub fn new(colour: Spectrum) -> Lambertian {
        Lambertian { colour: colour }
    }
}

impl BxDF for Lambertian {
    /// diffuse surfaces emit the same amount of light in all directions
    #[inline]
    fn f(&self, _: &Vector, _: &Vector) -> Spectrum {
        self.colour * consts::FRAC_1_PI
    }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_DIFFUSE | BSDF_REFLECTION
    }
}

/// A structure representing a Bidirectional Reflectance
/// Distribution Function for Specular Reflection.
/// This models the amount of incident
/// light reflected from a surface and in what direction(s).
pub struct SpecularReflection {
    r: Spectrum,
    // we store a trait object here as a reflective surface
    // can be something that has reflection and/or transmission
    // (e.g. metal or frosted glass)
    fresnel: Box<Fresnel>,
}

impl SpecularReflection {
    pub fn new<F: 'static + Fresnel>(r: Spectrum, fresnel: Box<F>) -> SpecularReflection {
        SpecularReflection {
            r: r,
            fresnel: fresnel as Box<Fresnel>,
        }
    }
}

impl BxDF for SpecularReflection {
    /// The Probability Distribution Function for use in
    /// Monte Carlo sampling.
    #[inline]
    fn pdf(&self, _: &Vector, _: &Vector) -> Pdf {
        0.0
    }

    fn sample_f(&self, wo: &Vector, _: Scalar, _: Scalar) -> (Spectrum, Vector, Pdf) {
        let wi = Vector::new(-wo.x, -wo.y, wo.z);
        let l = self.fresnel.evaluate(cos_theta(wo)).component_mul(&self.r) / cos_theta(&wi).abs();
        (l, wi, 1.0)
    }

    /// Specular reflection only produces light in a single direction
    /// given by the sample_f method
    #[inline]
    fn f(&self, _: &Vector, _: &Vector) -> Spectrum {
        na::zero()
    }

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
    t: Spectrum,
    etai: Scalar,
    etat: Scalar,
    fresnel: FresnelDielectric,
}

impl SpecularTransmission {
    pub fn new(t: Spectrum, etai: Scalar, etat: Scalar) -> SpecularTransmission {
        SpecularTransmission {
            t: t,
            etai: etai,
            etat: etat,
            fresnel: FresnelDielectric::new(etai, etat),
        }
    }
}

impl BxDF for SpecularTransmission {
    #[inline]
    fn pdf(&self, _: &Vector, _: &Vector) -> Pdf {
        0.0
    }

    fn sample_f(&self, wo: &Vector, _: Scalar, _: Scalar) -> (Spectrum, Vector, Pdf) {
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
            -Scalar::max(0.0, 1.0 - sint2)
        } else {
            Scalar::max(0.0, 1.0 - sint2)
        };

        let sint_over_sini = eta;
        let wi = Vector::new(sint_over_sini * -wo.x, sint_over_sini * -wo.y, cost);
        let f = self.fresnel.evaluate(cos_theta(wo));
        let transmitted = (Vector::new(1.0, 1.0, 1.0) - f).component_mul(&self.t) /
                          cos_theta(&wi).abs();
        (transmitted, wi, 1.0)
    }

    /// Specular transmission only produces light in a single direction
    /// given by the sample_f method.
    #[inline]
    fn f(&self, _: &Vector, _: &Vector) -> Spectrum {
        na::zero()
    }

    #[inline]
    fn bxdf_type(&self) -> BxDFType {
        BSDF_TRANSMISSION | BSDF_SPECULAR
    }
}

pub struct BSDF {
    normal: Vector,
    world_to_local: Rotation3<Scalar>,
    bxdfs: Vec<Box<BxDF>>,
}

impl BSDF {
    pub fn new(normal: Vector) -> BSDF {
        Self::new_with_bxdfs(normal, Vec::new())
    }

    pub fn new_with_bxdfs(normal: Vector, bxdfs: Vec<Box<BxDF>>) -> BSDF {
        BSDF {
            normal: normal,
            world_to_local: BSDF::world_to_local_from_normal(&normal),
            bxdfs: bxdfs,
        }
    }

    fn world_to_local_from_normal(normal: &Vector) -> Rotation3<Scalar> {
        let (tangent, binormal) = math::coordinate_system(normal);
        Rotation3::from_matrix_unchecked(Matrix3::new(tangent.x,
                                                        tangent.y,
                                                        tangent.z,
                                                        binormal.x,
                                                        binormal.y,
                                                        binormal.z,
                                                        normal.x,
                                                        normal.y,
                                                        normal.z))
    }

    #[inline]
    pub fn add_bxdf<T: 'static + BxDF>(&mut self, x: Box<T>) {
        self.bxdfs.push(x as Box<BxDF>);
    }

    #[inline]
    pub fn world_to_local(&self, v: &Vector) -> Vector {
        self.world_to_local.transform_vector(v)
    }

    #[inline]
    pub fn local_to_world(&self, v: &Vector) -> Vector {
        self.world_to_local.inverse_transform_vector(v)
    }

    pub fn sample_f<R>(&self,
                       wo_world: &Vector,
                       rng: &mut R,
                       flags: BxDFType)
                       -> (Spectrum, Vector, Pdf, Option<BxDFType>)
        where R: Rng
    {
        let wo = self.world_to_local(wo_world);

        let bxdfs: Vec<&Box<BxDF>> = self.bxdfs.iter().filter(|x| x.matches_flags(flags)).collect();
        // choose a random bxdf from the matching ones
        let bxdf = rng.choose(&bxdfs);

        match bxdf {
            Some(bxdf) => {
                let (u1, u2) = rng.gen::<(Scalar, Scalar)>();
                let (mut colour, wi, mut pdf) = bxdf.sample_f(&wo, u1, u2);
                let bxdf_type = bxdf.bxdf_type();

                let wi_world = self.local_to_world(&wi);

                // compute overall pdf with all matching BxDFs
                if !bxdf_type.intersects(BSDF_SPECULAR) && bxdfs.len() > 1 {
                    pdf = self.bxdfs.iter().filter(|x| x.matches_flags(flags)).map(|bxdf| bxdf.pdf(&wo, &wi)).sum();
                }
                let pdf = if bxdfs.len() > 1 {
                    pdf / bxdfs.len() as Scalar
                } else {
                    pdf
                };

                // compute value of BSDF in sampled direction
                if !bxdf_type.intersects(BSDF_SPECULAR) {
                    colour = na::zero();
                    let flags = if na::dot(&wi_world, &self.normal) *
                                   na::dot(wo_world, &self.normal) >
                                   0.0 {
                        // ignore BTDFs
                        flags - BSDF_TRANSMISSION
                    } else {
                        // ignore BRDFs
                        flags - BSDF_REFLECTION
                    };
                    for bxdf in self.bxdfs.iter().filter(|x| x.matches_flags(flags)) {
                        colour = colour + bxdf.f(&wo, &wi);
                    }
                }
                (colour, wi_world, pdf, Some(bxdf_type))
            }
            None => (na::zero(), na::zero(), 0.0, None),
        }
    }

    pub fn f(&self, wo_world: &Vector, wi_world: &Vector, flags: BxDFType) -> Spectrum {
        // incident and outgoing directions in local space
        let wi = self.world_to_local(wi_world);
        let wo = self.world_to_local(wo_world);

        let flags = {
            if na::dot(wo_world, &self.normal) * na::dot(wi_world, &self.normal) > 0.0 {
                // ignore BTDFs as the incident ray is on the outside of the surface
                flags - BSDF_TRANSMISSION
            } else {
                // ignore BRDFs as the incident ray is on the inside of the surface
                flags - BSDF_REFLECTION
            }
        };

        let mut f: Vector = na::zero();
        for bxdf in self.bxdfs.iter().filter(|x| x.matches_flags(flags)) {
            f = f + bxdf.f(&wi, &wo);
        }
        f
    }
}

/// Return the amount of energy reflected from a dielectric
/// surface (i.e. a non-conductor like glass).
fn fr_diel(cosi: Scalar, cost: Scalar, etai: &Spectrum, etat: &Spectrum) -> Spectrum {
    let rparl = ((*etat * cosi) - (*etai * cost)).component_div(&((*etat * cosi) + (*etai * cost)));
    let rperp = ((*etai * cosi) - (*etat * cost)).component_div(&((*etai * cosi) + (*etat * cost)));
    (rparl.component_mul(&rparl) + rperp.component_mul(&rperp)) / 2.0
}

/// Return the amount of energy reflected from a conductor.
fn fr_cond(cosi: Scalar, eta: &Spectrum, k: &Spectrum) -> Spectrum {
    let cosi_sq = cosi * cosi;
    let tmp = (eta.component_mul(eta) + k.component_mul(k)) * cosi_sq;
    let rparl2 = (tmp - (*eta * 2.0 * cosi) + Vector::from_element(1.0))
        .component_div(&(tmp + (*eta * 2.0 * cosi) + Vector::from_element(1.0)));
    let tmp_f = eta.component_mul(eta) + k.component_mul(k);
    let rperp2 = (tmp_f - (*eta * 2.0 * cosi) + Vector::from_element(cosi_sq))
        .component_div(&(tmp_f + (*eta * 2.0 * cosi) + Vector::from_element(cosi_sq)));
    (rparl2 + rperp2) / 2.0
}

pub trait Fresnel {
    fn evaluate(&self, cosi: Scalar) -> Spectrum;
}

pub struct FresnelConductor {
    eta: Spectrum,
    k: Spectrum,
}

impl FresnelConductor {
    pub fn new(eta: Spectrum, k: Spectrum) -> FresnelConductor {
        FresnelConductor { eta, k }
    }
}

impl Fresnel for FresnelConductor {
    #[inline]
    fn evaluate(&self, cosi: Scalar) -> Spectrum {
        fr_cond(cosi.abs(), &self.eta, &self.k)
    }
}

pub struct FresnelDielectric {
    etai: Scalar,
    etat: Scalar,
}

impl FresnelDielectric {
    pub fn new(etai: Scalar, etat: Scalar) -> FresnelDielectric {
        FresnelDielectric { etai, etat }
    }
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cosi: Scalar) -> Spectrum {
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
        let sint = etai / etat * Scalar::max(0.0, 1.0 - cosi * cosi).sqrt();
        if sint > 1.0 {
            // total internal reflection
            Vector::new(1.0, 1.0, 1.0)
        } else {
            let cost = Scalar::max(0.0, 1.0 - sint * sint).sqrt();
            fr_diel(cosi.abs(),
                    cost,
                    &Vector::new(etai, etai, etai),
                    &Vector::new(etat, etat, etat))
        }
    }
}
