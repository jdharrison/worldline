//! Standalone UDP server example — *not* part of the library core.
//!
//! Demonstrates how to drive `worldline`'s clock/engine from an external
//! process. Run with `cargo run --example server --features server -- --port 8080`.
//!
//! This file intentionally contains its own networking (tokio::net::UdpSocket)
//! so the `worldline` library stays pure.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use worldline::{
    simulation::Engine,
    time::{FidelityLevel, SimulationConfig},
};

#[derive(Parser, Debug)]
#[command(name = "worldline-server")]
#[command(about = "Worldline demo UDP server (proper-time simulation)")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    fidelity: Option<String>,

    #[arg(long)]
    steps_per_second: Option<u32>,

    #[arg(long)]
    time_multiplier: Option<f64>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    real_time_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerCommand {
    Start,
    Stop,
    Pause,
    Resume,
    Step,
    Status,
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerResponse {
    Status {
        state: String,
        simulation_time_ns: u64,
        proper_time_secs: f64,
        config: SimulationConfigResponse,
    },
    Ok {
        message: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimulationConfigResponse {
    target_steps_per_second: u32,
    simulation_time_multiplier: f64,
    fidelity: String,
    real_time_mode: bool,
}

impl From<&SimulationConfig> for SimulationConfigResponse {
    fn from(config: &SimulationConfig) -> Self {
        Self {
            target_steps_per_second: config.target_steps_per_second,
            simulation_time_multiplier: config.simulation_time_multiplier,
            fidelity: format!("{:?}", config.fidelity),
            real_time_mode: config.real_time_mode,
        }
    }
}

struct ServerState {
    engine: RwLock<Engine>,
    config: SimulationConfig,
}

impl ServerState {
    fn new(config: SimulationConfig) -> Self {
        Self {
            engine: RwLock::new(Engine::new(config)),
            config,
        }
    }

    async fn handle_command(&self, cmd: ServerCommand) -> ServerResponse {
        match cmd {
            ServerCommand::Start => {
                self.engine.write().await.start();
                ServerResponse::Ok {
                    message: "Simulation started".to_string(),
                }
            }
            ServerCommand::Stop => {
                self.engine.write().await.stop();
                ServerResponse::Ok {
                    message: "Simulation stopped".to_string(),
                }
            }
            ServerCommand::Pause => {
                self.engine.write().await.pause();
                ServerResponse::Ok {
                    message: "Simulation paused".to_string(),
                }
            }
            ServerCommand::Resume => {
                self.engine.write().await.resume();
                ServerResponse::Ok {
                    message: "Simulation resumed".to_string(),
                }
            }
            ServerCommand::Step => {
                let time = {
                    let mut e = self.engine.write().await;
                    e.step();
                    e.simulation_time_ns()
                };
                ServerResponse::Ok {
                    message: format!("Stepped to {} ns", time),
                }
            }
            ServerCommand::Status => {
                let e = self.engine.read().await;
                ServerResponse::Status {
                    state: format!("{:?}", e.state()),
                    simulation_time_ns: e.simulation_time_ns(),
                    proper_time_secs: e.proper_time_secs(),
                    config: SimulationConfigResponse::from(&self.config),
                }
            }
            ServerCommand::Reset => {
                self.engine.write().await.reset();
                ServerResponse::Ok {
                    message: "Simulation reset".to_string(),
                }
            }
        }
    }
}

fn parse_fidelity(s: &str) -> Result<FidelityLevel, String> {
    match s.to_lowercase().as_str() {
        "low" => Ok(FidelityLevel::Low),
        "medium" => Ok(FidelityLevel::Medium),
        "high" => Ok(FidelityLevel::High),
        "ultra" => Ok(FidelityLevel::Ultra),
        _ => Err(format!(
            "Invalid fidelity: {}. Valid: low, medium, high, ultra",
            s
        )),
    }
}

fn build_config(args: &Args) -> SimulationConfig {
    let mut config = SimulationConfig {
        real_time_mode: args.real_time_mode,
        ..Default::default()
    };

    if let Some(fidelity_str) = &args.fidelity {
        match parse_fidelity(fidelity_str) {
            Ok(fidelity) => {
                config.fidelity = fidelity;
                config.target_steps_per_second = fidelity.steps_per_second();
            }
            Err(e) => {
                warn!("Invalid fidelity: {}", e);
            }
        }
    }

    if let Some(steps) = args.steps_per_second {
        config.target_steps_per_second = steps;
    }

    if let Some(multiplier) = args.time_multiplier {
        config.simulation_time_multiplier = multiplier;
    }

    config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config = build_config(&args);

    info!("Starting worldline server with config: {:?}", config);

    let bind_addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
    let socket = UdpSocket::bind(bind_addr).await?;
    let local_addr = socket.local_addr()?;
    info!("UDP server listening on {}", local_addr);

    let server_state = Arc::new(ServerState::new(config));
    let mut buf = vec![0u8; 65535];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = buf[..len].to_vec();
        let state = Arc::clone(&server_state);
        let socket_clone = socket.local_addr().map(|_| ()).is_ok();
        // Handle inline (no extra spawn needed for this demo); keep select!-free.
        match serde_json::from_slice::<ServerCommand>(&data) {
            Ok(cmd) => {
                info!("Received command from {}: {:?}", addr, cmd);
                let response = state.handle_command(cmd).await;
                let response_bytes = serde_json::to_vec(&response)?;
                if let Err(e) = socket.send_to(&response_bytes, addr).await {
                    error!("Failed to send response to {}: {}", addr, e);
                }
            }
            Err(e) => {
                warn!("Failed to parse command from {}: {}", addr, e);
                let response = ServerResponse::Error {
                    message: format!("Invalid command: {}", e),
                };
                let response_bytes = serde_json::to_vec(&response)?;
                let _ = socket.send_to(&response_bytes, addr).await;
            }
        }
        let _ = socket_clone;
    }
}
