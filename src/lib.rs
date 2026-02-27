use tokio::time::Duration;

pub mod network;
pub mod simulation;
pub mod time;

pub use network::{NetworkConfig, NetworkRole, Packet, UdpChannel};
pub use simulation::{EngineState, SimulationEngine};
pub use time::{ClockState, FidelityLevel, SimulationClock, SimulationConfig, TimeStep};

#[derive(Debug, Clone)]
pub struct SimEngine {
    clock: SimulationClock,
    state: EngineState,
    config: SimulationConfig,
}

impl SimEngine {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            clock: SimulationClock::new(config),
            state: EngineState::Stopped,
            config,
        }
    }

    pub fn step(&mut self) -> Option<Duration> {
        if self.state != EngineState::Running {
            return None;
        }
        self.clock.advance()
    }

    pub fn simulation_time(&self) -> u64 {
        self.clock.simulation_time_ns()
    }

    pub fn state(&self) -> EngineState {
        self.state
    }

    pub fn set_state(&mut self, new_state: EngineState) {
        match new_state {
            EngineState::Running => self.clock.start(),
            EngineState::Paused => self.clock.pause(),
            EngineState::Stopped => self.clock.stop(),
            EngineState::Error => {}
        }
        self.state = new_state;
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }
}
