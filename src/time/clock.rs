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
    sim_time_ns: u64,
    wall_start: Instant,
    sim_start: Instant,
    last_step: Instant,
    total_steps: u64,
    accumulator_ns: u64,
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
        }
    }

    pub fn start(&mut self) {
        self.state = ClockState::Running;
        self.wall_start = Instant::now();
        self.sim_start = Instant::now();
        self.last_step = Instant::now();
    }

    pub fn pause(&mut self) {
        self.state = ClockState::Paused;
    }

    pub fn resume(&mut self) {
        self.state = ClockState::Running;
        self.last_step = Instant::now();
    }

    pub fn stop(&mut self) {
        self.state = ClockState::Stopped;
    }

    pub fn reset(&mut self) {
        self.sim_time_ns = 0;
        self.total_steps = 0;
        self.accumulator_ns = 0;
        self.last_step = Instant::now();
    }

    pub fn advance(&mut self) -> Option<Duration> {
        if self.state != ClockState::Running {
            return None;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_step);
        self.last_step = now;

        let target_step_ns = 1_000_000_000 / self.config.target_steps_per_second as u64;

        self.accumulator_ns += (elapsed.as_nanos() as u64
            * (self.config.simulation_time_multiplier * 1000.0) as u64)
            / 1000;

        let time_step_ns = if self.config.real_time_mode {
            target_step_ns
        } else {
            target_step_ns
        };

        if self.accumulator_ns >= time_step_ns {
            self.sim_time_ns += time_step_ns;
            self.accumulator_ns -= time_step_ns;
            self.total_steps += 1;
            Some(Duration::from_nanos(time_step_ns))
        } else {
            None
        }
    }

    pub fn simulation_time_ns(&self) -> u64 {
        self.sim_time_ns
    }

    pub fn wall_time_elapsed(&self) -> Duration {
        self.last_step.duration_since(self.wall_start)
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
}

pub type TimeStep = Duration;
