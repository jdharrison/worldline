use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FidelityLevel {
    Low,
    Medium,
    High,
    Ultra,
}

impl FidelityLevel {
    pub fn steps_per_second(&self) -> u32 {
        match self {
            FidelityLevel::Low => 10,
            FidelityLevel::Medium => 30,
            FidelityLevel::High => 60,
            FidelityLevel::Ultra => 120,
        }
    }

    pub fn max_entities(&self) -> usize {
        match self {
            FidelityLevel::Low => 100,
            FidelityLevel::Medium => 1000,
            FidelityLevel::High => 10000,
            FidelityLevel::Ultra => 50000,
        }
    }
}

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

    pub fn time_step(&self) -> Duration {
        Duration::from_nanos(1_000_000_000 / self.target_steps_per_second as u64)
    }
}

pub mod clock;

pub use clock::{ClockState, SimulationClock, TimeStep};
