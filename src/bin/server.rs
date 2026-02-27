use clap::Parser;
use serde::{Deserialize, Serialize};
use simengine::{
    simulation::SimulationEngine,
    time::{FidelityLevel, SimulationConfig},
    NetworkConfig, NetworkRole, UdpChannel,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "simengine-server")]
#[command(about = "Long-running simulation engine server")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    fidelity: Option<String>,

    #[arg(long, default_value = "60")]
    steps_per_second: Option<u32>,

    #[arg(long, default_value = "1.0")]
    time_multiplier: Option<f64>,

    #[arg(long, default_value = "false")]
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
        config: SimulationConfigResponse,
    },
    Ok { message: String },
    Error { message: String },
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
    engine: SimulationEngine,
}

impl ServerState {
    fn new(config: SimulationConfig) -> Self {
        Self {
            engine: SimulationEngine::new(config),
        }
    }

    async fn handle_command(&self, cmd: ServerCommand) -> ServerResponse {
        match cmd {
            ServerCommand::Start => {
                self.engine.start().await;
                ServerResponse::Ok {
                    message: "Simulation started".to_string(),
                }
            }
            ServerCommand::Stop => {
                self.engine.stop().await;
                ServerResponse::Ok {
                    message: "Simulation stopped".to_string(),
                }
            }
            ServerCommand::Pause => {
                self.engine.pause().await;
                ServerResponse::Ok {
                    message: "Simulation paused".to_string(),
                }
            }
            ServerCommand::Resume => {
                self.engine.resume().await;
                ServerResponse::Ok {
                    message: "Simulation resumed".to_string(),
                }
            }
            ServerCommand::Step => {
                self.engine.step().await;
                let time = self.engine.simulation_time_ns().await;
                ServerResponse::Ok {
                    message: format!("Stepped to {}", time),
                }
            }
            ServerCommand::Status => {
                let state = self.engine.state().await;
                let time = self.engine.simulation_time_ns().await;
                let config = self.engine.config();
                ServerResponse::Status {
                    state: format!("{:?}", state),
                    simulation_time_ns: time,
                    config: SimulationConfigResponse::from(config),
                }
            }
            ServerCommand::Reset => {
                self.engine.reset().await;
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
        _ => Err(format!("Invalid fidelity: {}. Valid: low, medium, high, ultra", s)),
    }
}

fn build_config(args: &Args) -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.real_time_mode = args.real_time_mode;

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

    info!("Starting simengine server with config: {:?}", config);

    let bind_addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
    let net_config = NetworkConfig {
        bind_address: bind_addr,
        remote_address: None,
        role: NetworkRole::Server,
        buffer_size: 65535,
    };

    let mut channel = UdpChannel::bind(net_config).await?;
    let local_addr = channel.local_addr()?;
    info!("UDP server listening on {}", local_addr);

    let server_state = Arc::new(RwLock::new(ServerState::new(config)));

    let mut buf = vec![0u8; 65535];

    loop {
        tokio::select! {
            result = channel.recv_from(&mut buf) => {
                match result {
                    Ok((len, addr)) => {
                        let data = &buf[..len];
                        match serde_json::from_slice::<ServerCommand>(data) {
                            Ok(cmd) => {
                                info!("Received command from {}: {:?}", addr, cmd);
                                let state = server_state.read().await;
                                let response = state.handle_command(cmd).await;
                                let response_bytes = serde_json::to_vec(&response)?;
                                if let Err(e) = channel.send_to(&response_bytes, addr).await {
                                    error!("Failed to send response to {}: {}", addr, e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse command from {}: {}", addr, e);
                                let response = ServerResponse::Error {
                                    message: format!("Invalid command: {}", e)
                                };
                                let response_bytes = serde_json::to_vec(&response)?;
                                let _ = channel.send_to(&response_bytes, addr).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("UDP recv error: {}", e);
                    }
                }
            }
        }
    }
}
