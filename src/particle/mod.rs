//! Particles with four-momentum and mass-shell condition.
//!
//! Hardened: constructors validate mass, velocity, finiteness; photon
//! handling explicitly affine vs proper-time; `try_*` variants return `Result`.

use crate::constants::{C, C2};
use crate::error::{Result, WorldlineError};
use crate::frame::Vec3;
use crate::spacetime::{Event, FourVector};
use crate::worldline::InertialWorldline;

/// A relativistic particle.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Rest mass in kg (0 for photon).
    pub mass: f64,
    /// Four-momentum `p = (E/c, px, py, pz)`.
    pub four_momentum: FourVector,
    /// Worldline of the particle (inertial for now; future: geodesic).
    pub worldline: InertialWorldline,
}

impl Particle {
    /// Create from rest mass and 3-velocity at `origin`.
    pub fn from_velocity(mass: f64, velocity: Vec3, origin: Event) -> Self {
        let beta2 = velocity.norm_squared() / C2;
        let gamma = if beta2 >= 1.0 {
            f64::INFINITY
        } else {
            (1.0 - beta2).powf(-0.5)
        };
        let energy = gamma * mass * C2;
        let px = gamma * mass * velocity.x;
        let py = gamma * mass * velocity.y;
        let pz = gamma * mass * velocity.z;
        let four_momentum = FourVector::new(energy / C, px, py, pz);
        let worldline = InertialWorldline::from_velocity(origin, velocity);
        Self {
            mass,
            four_momentum,
            worldline,
        }
    }

    pub fn try_from_velocity(mass: f64, velocity: Vec3, origin: Event) -> Result<Self> {
        if !mass.is_finite() || mass < 0.0 {
            return Err(WorldlineError::InvalidMass { mass });
        }
        velocity.check_finite("velocity")?;
        if !origin.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "origin".into(),
                value: origin.ct(),
            });
        }
        let beta = velocity.norm() / C;
        if beta >= 1.0 {
            return Err(WorldlineError::Superluminal {
                beta,
                velocity_norm: velocity.norm(),
            });
        }
        let p = Self::from_velocity(mass, velocity, origin);
        if !p.four_momentum.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "four_momentum".into(),
                value: p.four_momentum.ct,
            });
        }
        Ok(p)
    }

    /// Create photon from spatial momentum direction (null mass shell).
    pub fn photon(momentum: Vec3, origin: Event) -> Self {
        let p_norm = momentum.norm();
        let energy = p_norm * C;
        let four_momentum = FourVector::new(energy / C, momentum.x, momentum.y, momentum.z);
        let dir = if p_norm > 0.0 {
            Vec3::new(
                momentum.x / p_norm,
                momentum.y / p_norm,
                momentum.z / p_norm,
            )
        } else {
            Vec3::zero()
        };
        let vel = Vec3::new(dir.x * C, dir.y * C, dir.z * C);
        let four_vel = FourVector::new(C, vel.x, vel.y, vel.z);
        let worldline = InertialWorldline::new(origin, four_vel);
        Self {
            mass: 0.0,
            four_momentum,
            worldline,
        }
    }

    pub fn try_photon(momentum: Vec3, origin: Event) -> Result<Self> {
        momentum.check_finite("momentum")?;
        if !origin.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "origin".into(),
                value: origin.ct(),
            });
        }
        let p_norm = momentum.norm();
        if !p_norm.is_finite() {
            return Err(WorldlineError::NonFiniteInput {
                what: "p_norm".into(),
                value: p_norm,
            });
        }
        if p_norm == 0.0 {
            return Err(WorldlineError::InvalidSample {
                msg: "photon momentum norm zero".into(),
            });
        }
        Ok(Self::photon(momentum, origin))
    }

    pub fn energy(&self) -> f64 {
        self.four_momentum.ct * C
    }
    pub fn three_momentum(&self) -> Vec3 {
        Vec3::new(
            self.four_momentum.x,
            self.four_momentum.y,
            self.four_momentum.z,
        )
    }
    /// Invariant mass squared `m^2 c^4 = E^2 - p^2 c^2`.
    pub fn invariant_mass_squared(&self) -> f64 {
        let e = self.energy();
        let p2 = self.three_momentum().norm_squared();
        e * e - p2 * C2
    }
    /// Check mass shell within relative+absolute tolerance.
    pub fn is_on_shell(&self, tol: f64) -> bool {
        let expected = self.mass * self.mass * C2 * C2;
        let actual = self.invariant_mass_squared();
        // Relative for large masses, absolute for near-zero.
        let scale = expected.abs().max(1.0);
        (actual - expected).abs() <= tol * scale.max(tol)
    }
    pub fn is_on_shell_absolute(&self, atol: f64) -> bool {
        let expected = self.mass * self.mass * C2 * C2;
        (self.invariant_mass_squared() - expected).abs() < atol
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetime::Event;
    #[test]
    fn massive_particle_on_shell() {
        let origin = Event::at_origin();
        let v = Vec3::new(1e6, 0.0, 0.0);
        let p = Particle::from_velocity(9.11e-31, v, origin);
        assert!(p.is_on_shell(1e-6));
    }
    #[test]
    fn photon_null() {
        let origin = Event::at_origin();
        let ph = Particle::photon(Vec3::new(1e-27, 0.0, 0.0), origin);
        assert!((ph.invariant_mass_squared()).abs() < 1e-20);
    }
    #[test]
    fn try_superluminal_err() {
        let origin = Event::at_origin();
        assert!(Particle::try_from_velocity(1.0, Vec3::new(4e8, 0.0, 0.0), origin).is_err());
    }
}
