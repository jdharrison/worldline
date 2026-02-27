use std::sync::Arc;
use tokio::sync::RwLock;

use crate::time::{SimulationClock, SimulationConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Stopped,
    Running,
    Paused,
    Error,
}

pub struct SimulationEngine {
    clock: Arc<RwLock<SimulationClock>>,
    state: Arc<RwLock<EngineState>>,
    config: SimulationConfig,
}

impl SimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            clock: Arc::new(RwLock::new(SimulationClock::new(config.clone()))),
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

    pub async fn step(&self) {
        let mut clock = self.clock.write().await;
        clock.advance();
    }

    pub async fn simulation_time_ns(&self) -> u64 {
        self.clock.read().await.simulation_time_ns()
    }

    pub async fn state(&self) -> EngineState {
        self.state.read().await.clone()
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }
}
