//! Integrators for spacetime propagation — SR/GR ready.
//!
//! Provides a trait `Integrator` plus vetted implementations:
//! - `Leapfrog` (symplectic, energy-preserving for Newtonian)
//! - `RK4` (4th-order Runge-Kutta, generic)
//!
//! The interaction engine can plug any `SpacetimeMetric` and force model.
//! For solar-system J2000 validation, the lab should initialize states from
//! `astro::j2000_heliocentric` and step with `Epoch` advancing in TDB/TT.

use crate::error::Result;
use crate::spacetime::FourVector;

/// State for Newtonian-like integration: position (m) + velocity (m/s).
#[derive(Debug, Clone, Copy)]
pub struct NewtonianState {
    pub pos: crate::frame::Vec3,
    pub vel: crate::frame::Vec3,
    pub mass: f64,
}

impl NewtonianState {
    pub fn new(pos: crate::frame::Vec3, vel: crate::frame::Vec3, mass: f64) -> Self {
        Self { pos, vel, mass }
    }
}

/// Acceleration function: `a = f(state, t)` in m/s^2.
/// For GR scaffolding, this can include post-Newtonian corrections.
pub trait AccelerationModel: Send + Sync {
    fn acceleration(&self, state: &NewtonianState, t: f64) -> crate::frame::Vec3;
}

/// Newtonian gravity from a central GM (e.g. Sun).
#[derive(Debug, Clone, Copy)]
pub struct CentralGravity {
    pub gm: f64,
}

impl AccelerationModel for CentralGravity {
    fn acceleration(&self, state: &NewtonianState, _t: f64) -> crate::frame::Vec3 {
        let r2 = state.pos.norm_squared();
        if r2 < 1e-12 {
            return crate::frame::Vec3::zero();
        }
        let r = r2.sqrt();
        let factor = -self.gm / (r2 * r);
        state.pos * factor
    }
}

/// N-body gravity (heliocentric, point masses).
pub struct NBodyGravity {
    pub bodies: Vec<(crate::frame::Vec3, f64)>, // (pos, GM)
}

impl AccelerationModel for NBodyGravity {
    fn acceleration(&self, state: &NewtonianState, _t: f64) -> crate::frame::Vec3 {
        let mut acc = crate::frame::Vec3::zero();
        for (pos, gm) in &self.bodies {
            let dr = *pos - state.pos;
            let r2 = dr.norm_squared();
            if r2 < 1e6 {
                continue;
            }
            let r = r2.sqrt();
            let factor = gm / (r2 * r);
            acc = acc + dr * factor;
        }
        acc
    }
}

/// Generic integrator trait.
pub trait Integrator: Send + Sync {
    fn step(
        &self,
        state: &mut NewtonianState,
        t: f64,
        dt: f64,
        model: &dyn AccelerationModel,
    ) -> Result<()>;
}

/// Symplectic Leapfrog (velocity Verlet) — good for long-term energy.
#[derive(Debug, Clone, Copy, Default)]
pub struct Leapfrog;

impl Integrator for Leapfrog {
    fn step(
        &self,
        state: &mut NewtonianState,
        t: f64,
        dt: f64,
        model: &dyn AccelerationModel,
    ) -> Result<()> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(crate::error::WorldlineError::OutOfBounds {
                what: "dt".into(),
                value: dt,
                min: 1e-12,
                max: 1e12,
            });
        }
        // v_{1/2} = v + a(x)*dt/2
        let a0 = model.acceleration(state, t);
        state.vel = state.vel + a0 * (dt * 0.5);
        // x' = x + v_{1/2}*dt
        state.pos = state.pos + state.vel * dt;
        // a1 at new x
        let a1 = model.acceleration(state, t + dt);
        state.vel = state.vel + a1 * (dt * 0.5);
        Ok(())
    }
}

/// Classical RK4.
#[derive(Debug, Clone, Copy, Default)]
pub struct RK4;

impl Integrator for RK4 {
    fn step(
        &self,
        state: &mut NewtonianState,
        t: f64,
        dt: f64,
        model: &dyn AccelerationModel,
    ) -> Result<()> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(crate::error::WorldlineError::OutOfBounds {
                what: "dt".into(),
                value: dt,
                min: 1e-12,
                max: 1e12,
            });
        }
        let pos0 = state.pos;
        let vel0 = state.vel;

        let a1 = model.acceleration(state, t);
        let k1v = a1 * dt;
        let k1x = vel0 * dt;

        let s2 = NewtonianState::new(pos0 + k1x * 0.5, vel0 + k1v * 0.5, state.mass);
        let a2 = model.acceleration(&s2, t + dt * 0.5);
        let k2v = a2 * dt;
        let k2x = (vel0 + k1v * 0.5) * dt;

        let s3 = NewtonianState::new(pos0 + k2x * 0.5, vel0 + k2v * 0.5, state.mass);
        let a3 = model.acceleration(&s3, t + dt * 0.5);
        let k3v = a3 * dt;
        let k3x = (vel0 + k2v * 0.5) * dt;

        let s4 = NewtonianState::new(pos0 + k3x, vel0 + k3v, state.mass);
        let a4 = model.acceleration(&s4, t + dt);
        let k4v = a4 * dt;
        let k4x = (vel0 + k3v) * dt;

        state.pos = pos0 + (k1x + k2x * 2.0 + k3x * 2.0 + k4x) * (1.0 / 6.0);
        state.vel = vel0 + (k1v + k2v * 2.0 + k3v * 2.0 + k4v) * (1.0 / 6.0);
        Ok(())
    }
}

/// Proper-time vs coordinate-time correction for SR.
///
pub fn proper_time_step(dt_coordinate: f64, velocity: crate::frame::Vec3) -> Result<f64> {
    let beta = velocity.norm() * crate::constants::INV_C;
    if beta >= 1.0 {
        return Err(crate::error::WorldlineError::Superluminal {
            beta,
            velocity_norm: velocity.norm(),
        });
    }
    let gamma = (1.0 - beta * beta).powf(-0.5);
    Ok(dt_coordinate / gamma)
}

/// Four-vector from Newtonian state at time `t` (for worldline bridging).
pub fn four_vector_from_newtonian(state: &NewtonianState, t: f64) -> FourVector {
    let pos = state.pos;
    FourVector::new(crate::constants::C * t, pos.x, pos.y, pos.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{AU, GM_SUN};
    use crate::frame::Vec3;

    #[test]
    fn leapfrog_earth_like_orbit_stable() {
        // Circular orbit at 1 AU around Sun: v = sqrt(GM/r)
        let r = AU;
        let v_circ = (GM_SUN / r).sqrt();
        let mut state =
            NewtonianState::new(Vec3::new(r, 0.0, 0.0), Vec3::new(0.0, v_circ, 0.0), 5.97e24);
        let grav = CentralGravity { gm: GM_SUN };
        let leap = Leapfrog;
        let dt = 86400.0; // 1 day
        let steps = 365;
        let initial_r = state.pos.norm();
        for i in 0..steps {
            leap.step(&mut state, i as f64 * dt, dt, &grav).unwrap();
        }
        let final_r = state.pos.norm();
        // Leapfrog should preserve radius within ~1% over 1 year with 1-day steps.
        assert!(
            (final_r - initial_r).abs() / initial_r < 0.02,
            "r drift {} vs {}",
            final_r,
            initial_r
        );
    }

    #[test]
    fn rk4_earth_like_orbit_stable() {
        let r = AU;
        let v_circ = (GM_SUN / r).sqrt();
        let mut state =
            NewtonianState::new(Vec3::new(r, 0.0, 0.0), Vec3::new(0.0, v_circ, 0.0), 5.97e24);
        let grav = CentralGravity { gm: GM_SUN };
        let rk4 = RK4;
        let dt = 86400.0;
        for i in 0..10 {
            rk4.step(&mut state, i as f64 * dt, dt, &grav).unwrap();
        }
        // After 10 days, still near 1 AU
        let r_now = state.pos.norm();
        assert!((r_now - r).abs() / r < 0.001);
    }
}
