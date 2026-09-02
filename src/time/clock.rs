use super::SimulationConfig;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub struct SimulationClock {
    config: SimulationConfig,
    state: ClockState,
    /// Proper / simulation time in nanoseconds (monotonic).
    sim_time_ns: u64,
    wall_start: Instant,
    sim_start: Instant,
    last_step: Instant,
    total_steps: u64,
    accumulator_ns: u64,
    /// Optional paused elapsed to keep wall_time consistent.
    paused_elapsed: Option<Duration>,
}

impl SimulationClock {
    pub fn new(config: SimulationConfig) -> Self {
        let now = Instant::now();
        Self {
            config,
            state: ClockState::Stopped,
            sim_time_ns: 0,
            wall_start: now,
            sim_start: now,
            last_step: now,
            total_steps: 0,
            accumulator_ns: 0,
            paused_elapsed: None,
        }
    }

    pub fn start(&mut self) {
        let now = Instant::now();
        self.state = ClockState::Running;
        self.wall_start = now;
        self.sim_start = now;
        self.last_step = now;
        self.paused_elapsed = None;
        self.accumulator_ns = 0;
    }

    pub fn pause(&mut self) {
        if self.state == ClockState::Running {
            self.paused_elapsed = Some(Instant::now().duration_since(self.wall_start));
            self.state = ClockState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == ClockState::Paused {
            // Shift wall_start so wall_time_elapsed excludes paused duration.
            if let Some(paused) = self.paused_elapsed.take() {
                self.wall_start = Instant::now() - paused;
            }
            self.state = ClockState::Running;
            self.last_step = Instant::now();
        }
    }

    pub fn stop(&mut self) {
        self.state = ClockState::Stopped;
        self.paused_elapsed = None;
    }

    pub fn reset(&mut self) {
        self.sim_time_ns = 0;
        self.total_steps = 0;
        self.accumulator_ns = 0;
        self.last_step = Instant::now();
        self.paused_elapsed = None;
        if self.state == ClockState::Running {
            self.wall_start = Instant::now();
            self.sim_start = Instant::now();
        }
    }

    /// Advance one integration tick if enough wall-time has accumulated.
    ///
    /// Fixed vs prior: uses `f64` multiplier correctly (`elapsed * multiplier`)
    /// and respects `real_time_mode` — when false, always consumes a step
    /// (as-fast-as-possible / proper-time integration without wall pacing).
    pub fn advance(&mut self) -> Option<Duration> {
        if self.state != ClockState::Running {
            return None;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_step);
        self.last_step = now;

        let target_step_ns = 1_000_000_000 / self.config.target_steps_per_second as u64;

        if self.config.real_time_mode {
            // Properly scaled accumulator: sim_ns += wall_ns * multiplier
            let scaled =
                (elapsed.as_nanos() as f64 * self.config.simulation_time_multiplier) as u64;
            self.accumulator_ns = self.accumulator_ns.saturating_add(scaled);

            if self.accumulator_ns >= target_step_ns {
                self.sim_time_ns += target_step_ns;
                self.accumulator_ns -= target_step_ns;
                self.total_steps += 1;
                Some(Duration::from_nanos(target_step_ns))
            } else {
                None
            }
        } else {
            // As-fast-as-possible: one step per call, scaled step size.
            let scaled_step =
                (target_step_ns as f64 * self.config.simulation_time_multiplier) as u64;
            let step = scaled_step.max(1);
            self.sim_time_ns = self.sim_time_ns.saturating_add(step);
            self.total_steps += 1;
            Some(Duration::from_nanos(step))
        }
    }

    /// Deterministic advance for testing / batch integration without wall time.
    pub fn advance_by(&mut self, wall_delta: Duration) -> Option<Duration> {
        if self.state != ClockState::Running {
            return None;
        }
        let target_step_ns = 1_000_000_000 / self.config.target_steps_per_second as u64;
        if self.config.real_time_mode {
            let scaled =
                (wall_delta.as_nanos() as f64 * self.config.simulation_time_multiplier) as u64;
            self.accumulator_ns = self.accumulator_ns.saturating_add(scaled);
            if self.accumulator_ns >= target_step_ns {
                self.sim_time_ns += target_step_ns;
                self.accumulator_ns -= target_step_ns;
                self.total_steps += 1;
                Some(Duration::from_nanos(target_step_ns))
            } else {
                None
            }
        } else {
            let scaled_step =
                (target_step_ns as f64 * self.config.simulation_time_multiplier) as u64;
            let step = scaled_step.max(1);
            self.sim_time_ns = self.sim_time_ns.saturating_add(step);
            self.total_steps += 1;
            Some(Duration::from_nanos(step))
        }
    }

    pub fn simulation_time_ns(&self) -> u64 {
        self.sim_time_ns
    }

    /// Proper time in seconds (alias for simulation time in relativistic interpretation).
    pub fn proper_time_secs(&self) -> f64 {
        self.sim_time_ns as f64 * 1e-9
    }

    pub fn wall_time_elapsed(&self) -> Duration {
        match self.state {
            ClockState::Paused => self.paused_elapsed.unwrap_or(Duration::ZERO),
            _ => Instant::now().duration_since(self.wall_start),
        }
    }

    pub fn state(&self) -> ClockState {
        self.state
    }

    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    pub fn tick(&self) -> Duration {
        Duration::from_nanos(1_000_000_000 / self.config.target_steps_per_second as u64)
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }
}

pub type TimeStep = Duration;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::{FidelityLevel, SimulationConfig};
    use std::time::Duration;

    #[test]
    fn clock_advance_deterministic() {
        let cfg = SimulationConfig {
            target_steps_per_second: 10,
            simulation_time_multiplier: 1.0,
            fidelity: FidelityLevel::Low,
            real_time_mode: true,
        };
        let mut c = SimulationClock::new(cfg);
        c.start();
        // 100ms per step at 10 Hz. Feed 100ms -> one step.
        assert!(c.advance_by(Duration::from_millis(100)).is_some());
        assert_eq!(c.simulation_time_ns(), 100_000_000);
    }

    #[test]
    fn clock_multiplier_scales() {
        let cfg = SimulationConfig {
            target_steps_per_second: 10,
            simulation_time_multiplier: 2.0,
            fidelity: FidelityLevel::Low,
            real_time_mode: true,
        };
        let mut c = SimulationClock::new(cfg);
        c.start();
        // 50ms *2 =100ms -> one step
        assert!(c.advance_by(Duration::from_millis(50)).is_some());
    }

    #[test]
    fn clock_as_fast_as_possible() {
        let cfg = SimulationConfig {
            target_steps_per_second: 60,
            simulation_time_multiplier: 1.0,
            fidelity: FidelityLevel::Medium,
            real_time_mode: false,
        };
        let mut c = SimulationClock::new(cfg);
        c.start();
        assert!(c.advance().is_some());
    }
}
