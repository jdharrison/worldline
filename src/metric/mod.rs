//! Spacetime metric abstraction — SR with GR scaffolding.
//!
//! Hardened: interval helpers use relative tolerance for lightlike
//! classification; `proper_time_interval` validates sign and finite;
//! GR scaffolding now carries explicit error docs.

use crate::error::WorldlineError;
use crate::spacetime::FourVector;

/// Signature convention helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Signature {
    /// (+,-,-,-) — timelike intervals positive.
    #[default]
    MostlyMinus,
    /// (-,+,+,+) — spacelike intervals positive.
    MostlyPlus,
}

/// Abstract spacetime metric.
pub trait SpacetimeMetric: Send + Sync + std::fmt::Debug {
    fn signature(&self) -> Signature;
    fn dot(&self, a: &FourVector, b: &FourVector) -> f64;
    fn interval_squared(&self, delta: &FourVector) -> f64 {
        self.dot(delta, delta)
    }
    fn proper_time_interval(&self, delta: &FourVector) -> Option<f64> {
        if !delta.is_finite() {
            return None;
        }
        let s2 = self.interval_squared(delta);
        if !s2.is_finite() {
            return None;
        }
        match self.signature() {
            Signature::MostlyMinus => {
                if s2 < 0.0 {
                    None
                } else {
                    Some(s2.sqrt() * crate::constants::INV_C)
                }
            }
            Signature::MostlyPlus => {
                if s2 > 0.0 {
                    None
                } else {
                    Some((-s2).sqrt() * crate::constants::INV_C)
                }
            }
        }
    }
    /// Classify displacement with relative tolerance for lightlike.
    ///
    /// Uses `eps = 1e-12 * (|a||b| + |s2|)`.
    fn causal_character(&self, delta: &FourVector) -> CausalCharacter {
        let s2 = self.interval_squared(delta);
        if !s2.is_finite() || !delta.is_finite() {
            return CausalCharacter::Spacelike;
        }
        let norm_a =
            (delta.ct * delta.ct + delta.x * delta.x + delta.y * delta.y + delta.z * delta.z)
                .sqrt();
        let rel = 1e-12 * (norm_a * norm_a + s2.abs()).max(1.0);
        if s2.abs() < rel {
            CausalCharacter::Lightlike
        } else {
            match self.signature() {
                Signature::MostlyMinus => {
                    if s2 > 0.0 {
                        CausalCharacter::Timelike
                    } else {
                        CausalCharacter::Spacelike
                    }
                }
                Signature::MostlyPlus => {
                    if s2 < 0.0 {
                        CausalCharacter::Timelike
                    } else {
                        CausalCharacter::Spacelike
                    }
                }
            }
        }
    }
    fn check_finite(&self, v: &FourVector) -> Result<(), WorldlineError> {
        if !v.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "FourVector".into(),
                value: v.ct,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausalCharacter {
    Timelike,
    Lightlike,
    Spacelike,
}

/// Flat Minkowski metric.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinkowskiMetric {
    pub signature: Signature,
}
impl MinkowskiMetric {
    pub fn new(signature: Signature) -> Self {
        Self { signature }
    }
    pub fn mostly_minus() -> Self {
        Self {
            signature: Signature::MostlyMinus,
        }
    }
    pub fn mostly_plus() -> Self {
        Self {
            signature: Signature::MostlyPlus,
        }
    }
}
impl SpacetimeMetric for MinkowskiMetric {
    fn signature(&self) -> Signature {
        self.signature
    }
    fn dot(&self, a: &FourVector, b: &FourVector) -> f64 {
        match self.signature {
            Signature::MostlyMinus => a.ct * b.ct - a.x * b.x - a.y * b.y - a.z * b.z,
            Signature::MostlyPlus => -a.ct * b.ct + a.x * b.x + a.y * b.y + a.z * b.z,
        }
    }
}

/// Schwarzschild (static, spherically symmetric) metric — scaffolding.
///
/// At present delegates to Minkowski at infinity; position-dependent `g_tt`
/// and `g_rr` will land in a follow-up without breaking this type.
#[derive(Debug, Clone, Copy)]
pub struct SchwarzschildMetric {
    pub mass: f64,
    pub schwarzschild_radius: f64,
    pub signature: Signature,
}
impl SchwarzschildMetric {
    pub fn new(mass: f64) -> Self {
        let rs = 2.0 * crate::constants::G * mass * crate::constants::INV_C2;
        Self {
            mass,
            schwarzschild_radius: rs,
            signature: Signature::MostlyMinus,
        }
    }
    pub fn try_new(mass: f64) -> Result<Self, WorldlineError> {
        if !mass.is_finite() || mass < 0.0 {
            return Err(WorldlineError::InvalidMass { mass });
        }
        Ok(Self::new(mass))
    }
}
impl SpacetimeMetric for SchwarzschildMetric {
    fn signature(&self) -> Signature {
        self.signature
    }
    fn dot(&self, a: &FourVector, b: &FourVector) -> f64 {
        let mink = MinkowskiMetric::mostly_minus();
        mink.dot(a, b)
    }
}

/// Trait for metrics that can provide geodesic evolution.
pub trait GeodesicMetric: SpacetimeMetric {
    fn christoffel(&self, _event: &crate::spacetime::Event) -> [[[f64; 4]; 4]; 4] {
        [[[0.0; 4]; 4]; 4]
    }
}
impl GeodesicMetric for MinkowskiMetric {}
impl GeodesicMetric for SchwarzschildMetric {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetime::FourVector;
    #[test]
    fn minkowski_interval_timelike() {
        let m = MinkowskiMetric::mostly_minus();
        let d = FourVector::new(3e8, 0.0, 0.0, 0.0);
        assert_eq!(m.causal_character(&d), CausalCharacter::Timelike);
        assert!(m.proper_time_interval(&d).is_some());
    }
    #[test]
    fn minkowski_interval_spacelike() {
        let m = MinkowskiMetric::mostly_minus();
        let d = FourVector::new(0.0, 1.0, 0.0, 0.0);
        assert_eq!(m.causal_character(&d), CausalCharacter::Spacelike);
        assert!(m.proper_time_interval(&d).is_none());
    }
    #[test]
    fn lightlike_near_zero() {
        let m = MinkowskiMetric::mostly_minus();
        let d = FourVector::new(1.0, 1.0, 0.0, 0.0);
        assert_eq!(m.causal_character(&d), CausalCharacter::Lightlike);
    }
}
