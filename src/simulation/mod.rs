//! Simulation engine — worldline-aware, sync core with optional async wrapper.
//!
//! The sync [`Engine`] / [`SimEngine`] is the library primitive. An async
//! variant is available behind the `async` feature via `tokio::sync::RwLock`
//! to keep the core `worldline` crate free of a runtime dependency.

use crate::time::{SimulationClock, SimulationConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Stopped,
    Running,
    Paused,
    Error,
}

/// Synchronous worldline simulation engine (core).
///
/// Wraps a [`SimulationClock`] and tracks engine lifecycle. For relativistic
/// runs, `config.simulation_time_multiplier` can be set to `gamma` to map
/// proper time steps to coordinate time.
#[derive(Debug, Clone)]
pub struct Engine {
    clock: SimulationClock,
    state: EngineState,
    config: SimulationConfig,
}

impl Engine {
    pub fn new(config: SimulationConfig) -> Self {
        // Validate but keep construction infallible for ergonomics; caller can
        // call config.validate() explicitly. Debug builds assert.
        debug_assert!(config.validate().is_ok(), "invalid SimulationConfig");
        Self {
            clock: SimulationClock::new(config),
            state: EngineState::Stopped,
            config,
        }
    }

    /// Fallible constructor for interaction-engine import.
    pub fn try_new(config: SimulationConfig) -> crate::error::Result<Self> {
        config.validate()?;
        Ok(Self {
            clock: SimulationClock::new(config),
            state: EngineState::Stopped,
            config,
        })
    }

    pub fn start(&mut self) {
        self.clock.start();
        self.state = EngineState::Running;
    }

    pub fn stop(&mut self) {
        self.clock.stop();
        self.state = EngineState::Stopped;
    }

    pub fn pause(&mut self) {
        self.clock.pause();
        self.state = EngineState::Paused;
    }

    pub fn resume(&mut self) {
        self.clock.resume();
        self.state = EngineState::Running;
    }

    pub fn reset(&mut self) {
        self.clock.reset();
        self.state = EngineState::Stopped;
    }

    /// Advance one tick if running. Returns the proper-time step if one was taken.
    pub fn step(&mut self) -> Option<std::time::Duration> {
        if self.state != EngineState::Running {
            return None;
        }
        self.clock.advance()
    }

    /// Deterministic step for testing / batch integration.
    pub fn step_by(&mut self, wall_delta: std::time::Duration) -> Option<std::time::Duration> {
        if self.state != EngineState::Running {
            return None;
        }
        self.clock.advance_by(wall_delta)
    }

    pub fn simulation_time_ns(&self) -> u64 {
        self.clock.simulation_time_ns()
    }

    pub fn proper_time_secs(&self) -> f64 {
        self.clock.proper_time_secs()
    }

    pub fn state(&self) -> EngineState {
        self.state
    }

    pub fn set_state(&mut self, new_state: EngineState) {
        match new_state {
            EngineState::Running => {
                if self.state == EngineState::Paused {
                    self.clock.resume();
                } else {
                    self.clock.start();
                }
            }
            EngineState::Paused => self.clock.pause(),
            EngineState::Stopped => self.clock.stop(),
            EngineState::Error => {}
        }
        self.state = new_state;
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }

    pub fn clock(&self) -> &SimulationClock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut SimulationClock {
        &mut self.clock
    }
}

// Keep old name as alias for migration.
pub type SimEngine = Engine;

/// Re-export for doc compatibility.
pub use Engine as SimulationEngineSync;

// ---- Optional async engine (feature = "async") ----

#[cfg(feature = "async")]
mod async_engine {
    use super::{EngineState, SimulationClock, SimulationConfig};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    pub struct SimulationEngine {
        clock: Arc<RwLock<SimulationClock>>,
        state: Arc<RwLock<EngineState>>,
        config: SimulationConfig,
    }

    impl SimulationEngine {
        pub fn new(config: SimulationConfig) -> Self {
            Self {
                clock: Arc::new(RwLock::new(SimulationClock::new(config))),
                state: Arc::new(RwLock::new(EngineState::Stopped)),
                config,
            }
        }

        pub async fn start(&self) {
            let mut clock = self.clock.write().await;
            clock.start();
            *self.state.write().await = EngineState::Running;
        }

        pub async fn stop(&self) {
            let mut clock = self.clock.write().await;
            clock.stop();
            *self.state.write().await = EngineState::Stopped;
        }

        pub async fn pause(&self) {
            let mut clock = self.clock.write().await;
            clock.pause();
            *self.state.write().await = EngineState::Paused;
        }

        pub async fn resume(&self) {
            let mut clock = self.clock.write().await;
            clock.resume();
            *self.state.write().await = EngineState::Running;
        }

        pub async fn reset(&self) {
            let mut clock = self.clock.write().await;
            clock.reset();
            *self.state.write().await = EngineState::Stopped;
        }

        pub async fn step(&self) -> Option<std::time::Duration> {
            let mut clock = self.clock.write().await;
            clock.advance()
        }

        pub async fn simulation_time_ns(&self) -> u64 {
            self.clock.read().await.simulation_time_ns()
        }

        pub async fn state(&self) -> EngineState {
            *self.state.read().await
        }

        pub fn config(&self) -> &SimulationConfig {
            &self.config
        }
    }
}

#[cfg(feature = "async")]
pub use async_engine::SimulationEngine;
