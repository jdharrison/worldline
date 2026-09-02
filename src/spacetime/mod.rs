//! Spacetime primitives: [`FourVector`], [`Event`], intervals.
//!
//! Hardened: constructors validate finite inputs; interval helpers use
//! metric-aware relative tolerances; `Event` keeps semantic separation
//! from raw `FourVector`.

use crate::constants::{C, INV_C};
use crate::error::{Result, WorldlineError};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};

/// A four-vector in spacetime / energy-momentum space.
///
/// Stored as `(ct, x, y, z)` where `ct` has units of meters.
/// For four-momentum the same layout is `(E/c, px, py, pz)`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FourVector {
    pub ct: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl FourVector {
    pub fn new(ct: f64, x: f64, y: f64, z: f64) -> Self {
        Self { ct, x, y, z }
    }

    pub fn try_new(ct: f64, x: f64, y: f64, z: f64) -> Result<Self> {
        for (name, v) in [("ct", ct), ("x", x), ("y", y), ("z", z)] {
            if !v.is_finite() {
                return Err(WorldlineError::NonFiniteInput {
                    what: name.to_string(),
                    value: v,
                });
            }
        }
        Ok(Self { ct, x, y, z })
    }

    /// Create from coordinate time `t` (seconds) — converts to `ct`.
    pub fn from_event(t: f64, x: f64, y: f64, z: f64) -> Self {
        Self { ct: C * t, x, y, z }
    }

    pub fn try_from_event(t: f64, x: f64, y: f64, z: f64) -> Result<Self> {
        if !t.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "t".into(),
                value: t,
            });
        }
        Self::try_new(C * t, x, y, z)
    }

    pub fn zero() -> Self {
        Self {
            ct: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn spatial_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn spatial_norm(&self) -> f64 {
        self.spatial_squared().sqrt()
    }

    pub fn as_array(&self) -> [f64; 4] {
        [self.ct, self.x, self.y, self.z]
    }

    pub fn is_finite(&self) -> bool {
        self.ct.is_finite() && self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn check_finite(&self) -> Result<()> {
        if !self.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "FourVector".into(),
                value: self.ct,
            });
        }
        Ok(())
    }
}

impl Add for FourVector {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            ct: self.ct + rhs.ct,
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl Sub for FourVector {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            ct: self.ct - rhs.ct,
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl Neg for FourVector {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            ct: -self.ct,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
impl Mul<f64> for FourVector {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self {
            ct: self.ct * s,
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}
impl Mul<FourVector> for f64 {
    type Output = FourVector;
    fn mul(self, v: FourVector) -> FourVector {
        v * self
    }
}

/// An event in spacetime — a point with coordinates `(ct, x, y, z)`.
///
/// Semantically distinct from a generic [`FourVector`] (displacement or
/// momentum) but representation-identical. Newtype keeps APIs self-documenting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Event(pub FourVector);

impl Event {
    pub fn new(ct: f64, x: f64, y: f64, z: f64) -> Self {
        Self(FourVector::new(ct, x, y, z))
    }
    pub fn try_new(ct: f64, x: f64, y: f64, z: f64) -> Result<Self> {
        Ok(Self(FourVector::try_new(ct, x, y, z)?))
    }
    pub fn from_t(t: f64, x: f64, y: f64, z: f64) -> Self {
        Self(FourVector::from_event(t, x, y, z))
    }
    pub fn try_from_t(t: f64, x: f64, y: f64, z: f64) -> Result<Self> {
        Ok(Self(FourVector::try_from_event(t, x, y, z)?))
    }
    pub fn at_origin() -> Self {
        Self(FourVector::zero())
    }
    pub fn ct(&self) -> f64 {
        self.0.ct
    }
    pub fn t(&self) -> f64 {
        self.0.ct * INV_C
    }
    pub fn x(&self) -> f64 {
        self.0.x
    }
    pub fn y(&self) -> f64 {
        self.0.y
    }
    pub fn z(&self) -> f64 {
        self.0.z
    }
    pub fn displacement_to(&self, other: &Self) -> FourVector {
        other.0 - self.0
    }
    pub fn is_finite(&self) -> bool {
        self.0.is_finite()
    }
}

impl From<FourVector> for Event {
    fn from(v: FourVector) -> Self {
        Self(v)
    }
}
impl From<Event> for FourVector {
    fn from(e: Event) -> Self {
        e.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_vector_add() {
        let a = FourVector::new(1.0, 2.0, 3.0, 4.0);
        let b = FourVector::new(5.0, 6.0, 7.0, 8.0);
        assert_eq!(a + b, FourVector::new(6.0, 8.0, 10.0, 12.0));
    }
    #[test]
    fn event_t_conversion() {
        let e = Event::from_t(1.0, 0.0, 0.0, 0.0);
        assert!((e.ct() - C).abs() < 1e-6);
        assert!((e.t() - 1.0).abs() < 1e-12);
    }
    #[test]
    fn try_new_non_finite_err() {
        assert!(FourVector::try_new(f64::NAN, 0.0, 0.0, 0.0).is_err());
        assert!(Event::try_new(0.0, f64::INFINITY, 0.0, 0.0).is_err());
    }
}
