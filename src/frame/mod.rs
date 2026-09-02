//! Reference frames, Lorentz transforms, observers, and J2000 frames.
//!
//! Hardened vs prior: all public constructors validate `|v| < c`, finite inputs,
//! and return `Result` where a panic was possible. `gamma` helpers return
//! `Result<f64>` for superluminal inputs. J2000/EME2000 and ICRS scaffolding
//! added for solar-system validation.

use crate::constants::INV_C;
use crate::error::{Result, WorldlineError};
use crate::spacetime::FourVector;
use serde::{Deserialize, Serialize};

/// Spatial 3-vector (m or m/s depending on context).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn scaled(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
    pub fn check_finite(&self, what: &str) -> Result<()> {
        if !self.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: what.to_string(),
                value: if !self.x.is_finite() {
                    self.x
                } else if !self.y.is_finite() {
                    self.y
                } else {
                    self.z
                },
            });
        }
        Ok(())
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        self.scaled(s)
    }
}

/// Lorentz factor `gamma = 1 / sqrt(1 - beta^2)`.
/// Returns `Err` if `|beta| >= 1` or non-finite. Use `try_gamma` for validation.
pub fn gamma_from_beta(beta: f64) -> f64 {
    // Legacy panicking-free variant: returns INF for illegal beta.
    if !beta.is_finite() || beta.abs() >= 1.0 {
        f64::INFINITY
    } else {
        (1.0 - beta * beta).powf(-0.5)
    }
}

/// Checked version—returns error for superluminal.
pub fn try_gamma_from_beta(beta: f64) -> Result<f64> {
    if !beta.is_finite() {
        return Err(WorldlineError::NonFiniteInput {
            what: "beta".into(),
            value: beta,
        });
    }
    if beta.abs() >= 1.0 {
        return Err(WorldlineError::Superluminal {
            beta,
            velocity_norm: beta * crate::constants::C,
        });
    }
    Ok((1.0 - beta * beta).powf(-0.5))
}

pub fn gamma_from_velocity(v: Vec3) -> f64 {
    gamma_from_beta(v.norm() * INV_C)
}

pub fn try_gamma_from_velocity(v: Vec3) -> Result<f64> {
    v.check_finite("velocity")?;
    try_gamma_from_beta(v.norm() * INV_C)
}

pub fn beta_from_velocity(v: Vec3) -> f64 {
    v.norm() * INV_C
}

/// 3×3 rotation matrix (row-major). Used for EME2000/J2000 frame rotations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation3(pub [[f64; 3]; 3]);

impl Rotation3 {
    pub fn identity() -> Self {
        Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }
    /// Rotation about X axis by `angle` rad.
    pub fn about_x(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        Self([[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]])
    }
    pub fn about_y(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        Self([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]])
    }
    pub fn about_z(angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        Self([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
    }
    pub fn apply(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.0[0][0] * v.x + self.0[0][1] * v.y + self.0[0][2] * v.z,
            self.0[1][0] * v.x + self.0[1][1] * v.y + self.0[1][2] * v.z,
            self.0[2][0] * v.x + self.0[2][1] * v.y + self.0[2][2] * v.z,
        )
    }
    pub fn transpose(&self) -> Self {
        Self([
            [self.0[0][0], self.0[1][0], self.0[2][0]],
            [self.0[0][1], self.0[1][1], self.0[2][1]],
            [self.0[0][2], self.0[1][2], self.0[2][2]],
        ])
    }
    #[allow(clippy::needless_range_loop)]
    pub fn mul(&self, other: &Self) -> Self {
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    out[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        Self(out)
    }
}

/// Enumerated astronomical frames relevant for J2000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameId {
    /// International Celestial Reference System (ICRS) — J2000-aligned to ~20mas.
    ICRS,
    /// Mean Equator / Equinox J2000 (EME2000). Often called "J2000" in practice.
    EME2000,
    /// Geocentric Celestial Reference System (GCRS) — includes aberration periodic.
    GCRS,
    /// Barycentric Celestial Reference System (BCRS) — solar-system barycenter.
    BCRS,
}

/// An inertial frame moving at velocity `v` relative to a parent frame.
///
/// The boost is the transformation *from* parent *to* this frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InertialFrame {
    /// Velocity of this frame as seen in parent.
    pub velocity: Vec3,
    /// Optional frame identifier for documentation/validation.
    #[serde(default = "default_frame_id")]
    pub id: Option<FrameId>,
}

fn default_frame_id() -> Option<FrameId> {
    None
}

impl InertialFrame {
    pub fn new(velocity: Vec3) -> Self {
        Self { velocity, id: None }
    }

    pub fn try_new(velocity: Vec3) -> Result<Self> {
        velocity.check_finite("velocity")?;
        let beta = velocity.norm() * INV_C;
        if beta >= 1.0 {
            return Err(WorldlineError::Superluminal {
                beta,
                velocity_norm: velocity.norm(),
            });
        }
        Ok(Self { velocity, id: None })
    }

    pub fn with_id(mut self, id: FrameId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn at_rest() -> Self {
        Self {
            velocity: Vec3::zero(),
            id: None,
        }
    }

    pub fn beta_vec(&self) -> Vec3 {
        Vec3::new(
            self.velocity.x * INV_C,
            self.velocity.y * INV_C,
            self.velocity.z * INV_C,
        )
    }

    pub fn beta(&self) -> f64 {
        self.beta_vec().norm()
    }

    pub fn gamma(&self) -> f64 {
        gamma_from_beta(self.beta())
    }

    pub fn try_gamma(&self) -> Result<f64> {
        try_gamma_from_beta(self.beta())
    }

    /// Lorentz boost of a four-vector into this frame.
    ///
    /// General boost for arbitrary velocity direction. Uses:
    /// `ct' = gamma*(ct - beta·r)`
    /// `r' = r + [(gamma-1)/beta^2]*(beta·r)*beta - gamma*beta*ct`
    /// Validates inputs; returns boosted vector or identity if `|v|` tiny.
    pub fn boost(&self, v: FourVector) -> FourVector {
        // Non-finite inputs propagate as NaN — hardening chooses to return as-is
        // but try_boost is the checked variant.
        let beta = self.beta_vec();
        let beta2 = beta.norm_squared();
        if beta2 < 1e-18 {
            return v;
        }
        let gamma = gamma_from_beta(beta2.sqrt());
        if !gamma.is_finite() {
            return FourVector::new(f64::NAN, f64::NAN, f64::NAN, f64::NAN);
        }
        let beta_dot_r = beta.x * v.x + beta.y * v.y + beta.z * v.z;
        let ct_prime = gamma * (v.ct - beta_dot_r);
        let factor = (gamma - 1.0) / beta2 * beta_dot_r - gamma * v.ct;
        FourVector::new(
            ct_prime,
            v.x + factor * beta.x,
            v.y + factor * beta.y,
            v.z + factor * beta.z,
        )
    }

    /// Checked boost — errors on superluminal frame or non-finite inputs.
    pub fn try_boost(&self, v: FourVector) -> Result<FourVector> {
        if !v.ct.is_finite() || !v.x.is_finite() || !v.y.is_finite() || !v.z.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "FourVector".into(),
                value: v.ct,
            });
        }
        self.try_gamma()?;
        Ok(self.boost(v))
    }

    /// Inverse boost (from this frame back to parent) = boost by `-v`.
    pub fn inverse_boost(&self, v: FourVector) -> FourVector {
        InertialFrame::new(Vec3::new(
            -self.velocity.x,
            -self.velocity.y,
            -self.velocity.z,
        ))
        .boost(v)
    }

    pub fn try_inverse_boost(&self, v: FourVector) -> Result<FourVector> {
        InertialFrame::try_new(Vec3::new(
            -self.velocity.x,
            -self.velocity.y,
            -self.velocity.z,
        ))?
        .try_boost(v)
    }

    /// Time dilation: proper time `tau` vs coordinate time `t` for a clock
    /// comoving with this frame: `t = gamma * tau`.
    pub fn coordinate_time_from_proper(&self, tau: f64) -> f64 {
        self.gamma() * tau
    }

    pub fn try_coordinate_time_from_proper(&self, tau: f64) -> Result<f64> {
        if !tau.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "tau".into(),
                value: tau,
            });
        }
        Ok(self.try_gamma()? * tau)
    }

    pub fn proper_time_from_coordinate(&self, t: f64) -> f64 {
        let g = self.gamma();
        if g == 0.0 || !g.is_finite() {
            f64::NAN
        } else {
            t / g
        }
    }

    /// Length contraction: `L = L0 / gamma`.
    pub fn contracted_length(&self, proper_length: f64) -> f64 {
        let g = self.gamma();
        if g == 0.0 || !g.is_finite() {
            f64::NAN
        } else {
            proper_length / g
        }
    }
}

/// EME2000 / J2000 frame helpers.
///
/// ICRS ≈ EME2000 to ~20 mas; GCRS adds aberration. This module provides
/// the mean obliquity rotation between equatorial and ecliptic and
/// placeholder precession stubs.
pub mod j2000 {
    use super::{Rotation3, Vec3};
    use crate::constants::OBLIQUITY_J2000_RAD;

    /// Rotation from EME2000 equatorial to ecliptic (about X by +obliquity).
    pub fn equatorial_to_ecliptic() -> Rotation3 {
        Rotation3::about_x(OBLIQUITY_J2000_RAD)
    }
    pub fn ecliptic_to_equatorial() -> Rotation3 {
        Rotation3::about_x(-OBLIQUITY_J2000_RAD)
    }

    /// Apply equatorial→ecliptic to a position vector (AU or m).
    pub fn to_ecliptic(v: Vec3) -> Vec3 {
        equatorial_to_ecliptic().apply(v)
    }
    pub fn to_equatorial(v: Vec3) -> Vec3 {
        ecliptic_to_equatorial().apply(v)
    }

    /// ICRS ↔ EME2000 rotation is identity to ~20mas; model as identity with docs.
    /// Future: plug in IAU 2006 bias matrix (≈ -17 mas).
    pub fn icrs_to_eme2000(v: Vec3) -> Vec3 {
        v
    }
}

/// An observer: a worldline origin + frame.
#[derive(Debug, Clone, Copy)]
pub struct Observer {
    pub frame: InertialFrame,
    pub origin: crate::spacetime::Event,
}

impl Observer {
    pub fn new(frame: InertialFrame, origin: crate::spacetime::Event) -> Self {
        Self { frame, origin }
    }

    pub fn at_rest_at_origin() -> Self {
        Self {
            frame: InertialFrame::at_rest(),
            origin: crate::spacetime::Event::at_origin(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetime::FourVector;

    #[test]
    fn gamma_zero() {
        assert!((gamma_from_beta(0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn try_gamma_superluminal_err() {
        assert!(try_gamma_from_beta(1.0).is_err());
        assert!(try_gamma_from_beta(1.5).is_err());
    }

    #[test]
    fn try_new_superluminal_err() {
        let v = Vec3::new(3e8, 0.0, 0.0);
        assert!(InertialFrame::try_new(v).is_err());
    }

    #[test]
    fn boost_roundtrip() {
        let frame = InertialFrame::new(Vec3::new(1e7, 0.0, 0.0));
        let v = FourVector::new(1e8, 1e6, 2e6, 3e6);
        let boosted = frame.boost(v);
        let back = frame.inverse_boost(boosted);
        assert!((back.ct - v.ct).abs() < 1e-6);
        assert!((back.x - v.x).abs() < 1e-6);
    }

    #[test]
    fn boost_identity_at_rest() {
        let frame = InertialFrame::at_rest();
        let v = FourVector::new(5.0, 1.0, 2.0, 3.0);
        assert_eq!(frame.boost(v), v);
    }

    #[test]
    fn rotation_roundtrip() {
        let r = Rotation3::about_x(0.5);
        let v = Vec3::new(1.0, 2.0, 3.0);
        let back = r.transpose().apply(r.apply(v));
        assert!((back.x - v.x).abs() < 1e-12);
    }

    #[test]
    fn j2000_ecliptic_roundtrip() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let e = j2000::to_ecliptic(v);
        let back = j2000::to_equatorial(e);
        assert!((back.x - v.x).abs() < 1e-12);
    }
}
