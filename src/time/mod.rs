//! Simulation timing — reinterpreted for relativistic worldlines.
//!
//! `FidelityLevel` is now integration fidelity (steps per proper-time
//! second) rather than generic entity budget. `SimulationConfig` cleanly
//! separates **coordinate time** (lab frame) from **proper time** (clock
//! comoving with a worldline) via `simulation_time_multiplier` (which can
//! be driven by a Lorentz factor) and `real_time_mode` pacing.
//!
//! The underlying [`clock::SimulationClock`] is intentionally still
//! wall-time-based for pacing, but now also tracks proper vs coordinate
//! time consistently and fixes prior bugs (float truncation, wall clock).

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FidelityLevel {
    Low,
    Medium,
    High,
    Ultra,
}

impl FidelityLevel {
    /// Integration steps per proper-time second. Higher fidelity -> finer
    /// worldline sampling / smaller truncation error.
    pub fn steps_per_second(&self) -> u32 {
        match self {
            FidelityLevel::Low => 10,
            FidelityLevel::Medium => 30,
            FidelityLevel::High => 60,
            FidelityLevel::Ultra => 120,
        }
    }

    /// Max worldlines / entities budget for this fidelity.
    pub fn max_entities(&self) -> usize {
        match self {
            FidelityLevel::Low => 100,
            FidelityLevel::Medium => 1000,
            FidelityLevel::High => 10000,
            FidelityLevel::Ultra => 50000,
        }
    }

    /// Suggested proper-time step `dtau` for this fidelity at `c` scaling.
    pub fn proper_time_step(&self) -> Duration {
        Duration::from_nanos(1_000_000_000 / self.steps_per_second() as u64)
    }
}

/// Configuration for the simulation clock.
///
/// - `target_steps_per_second` is the integration rate in proper time.
/// - `simulation_time_multiplier` is a time-dilation / fast-forward factor:
///   `simulation_time = wall_time * multiplier`. For relativistic runs
///   set this to `gamma` to map proper -> coordinate time.
/// - `real_time_mode` when true paces to wall time; when false the
///   simulation runs as-fast-as-possible (useful for batch proper-time
///   integration).
#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub target_steps_per_second: u32,
    pub simulation_time_multiplier: f64,
    pub fidelity: FidelityLevel,
    pub real_time_mode: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            target_steps_per_second: 60,
            simulation_time_multiplier: 1.0,
            fidelity: FidelityLevel::Medium,
            real_time_mode: true,
        }
    }
}

impl SimulationConfig {
    pub fn with_fidelity(mut self, fidelity: FidelityLevel) -> Self {
        self.target_steps_per_second = fidelity.steps_per_second();
        self.fidelity = fidelity;
        self
    }

    pub fn with_time_multiplier(mut self, m: f64) -> Self {
        self.simulation_time_multiplier = m;
        self
    }

    /// Set multiplier from a Lorentz factor `gamma` (proper -> coordinate).
    pub fn with_gamma(mut self, gamma: f64) -> Self {
        self.simulation_time_multiplier = gamma;
        self
    }

    /// Validate config; errors on non-finite multiplier, zero steps, etc.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.target_steps_per_second == 0 {
            return Err(crate::error::WorldlineError::OutOfBounds {
                what: "target_steps_per_second".into(),
                value: 0.0,
                min: 1.0,
                max: 1_000_000.0,
            });
        }
        if self.target_steps_per_second > 1_000_000 {
            return Err(crate::error::WorldlineError::OutOfBounds {
                what: "target_steps_per_second".into(),
                value: self.target_steps_per_second as f64,
                min: 1.0,
                max: 1_000_000.0,
            });
        }
        if !self.simulation_time_multiplier.is_finite() {
            return Err(crate::error::WorldlineError::NonFiniteInput {
                what: "simulation_time_multiplier".into(),
                value: self.simulation_time_multiplier,
            });
        }
        if self.simulation_time_multiplier <= 0.0 {
            return Err(crate::error::WorldlineError::OutOfBounds {
                what: "simulation_time_multiplier".into(),
                value: self.simulation_time_multiplier,
                min: f64::EPSILON,
                max: 1e12,
            });
        }
        if self.simulation_time_multiplier > 1e9 {
            return Err(crate::error::WorldlineError::OutOfBounds {
                what: "simulation_time_multiplier".into(),
                value: self.simulation_time_multiplier,
                min: f64::EPSILON,
                max: 1e9,
            });
        }
        Ok(())
    }

    pub fn try_with_fidelity(mut self, fidelity: FidelityLevel) -> crate::error::Result<Self> {
        self.target_steps_per_second = fidelity.steps_per_second();
        self.fidelity = fidelity;
        self.validate().map(|_| self)
    }

    pub fn time_step(&self) -> Duration {
        Duration::from_nanos(1_000_000_000 / self.target_steps_per_second as u64)
    }

    pub fn proper_time_step(&self) -> Duration {
        self.time_step()
    }
}

pub mod clock;
pub mod epoch;

pub use clock::{ClockState, SimulationClock, TimeStep};
pub use epoch::{Epoch, TimeScale};
