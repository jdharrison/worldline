use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket as TokioUdpSocket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkRole {
    Server,
    Client,
    Peer,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: SocketAddr,
    pub remote_address: Option<SocketAddr>,
    pub role: NetworkRole,
    pub buffer_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:0".parse().unwrap(),
            remote_address: None,
            role: NetworkRole::Peer,
            buffer_size: 65535,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub simulation_time: u64,
    pub sequence: u64,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(simulation_time: u64, sequence: u64, payload: Vec<u8>) -> Self {
        Self {
            simulation_time,
            sequence,
            payload,
        }
    }
}

pub struct UdpChannel {
    socket: TokioUdpSocket,
    config: NetworkConfig,
    sequence: u64,
}

impl UdpChannel {
    pub async fn bind(config: NetworkConfig) -> Result<Self, std::io::Error> {
        let socket = TokioUdpSocket::bind(config.bind_address).await?;

        Ok(Self {
            socket,
            config,
            sequence: 0,
        })
    }

    pub async fn send_to(
        &mut self,
        data: &[u8],
        addr: SocketAddr,
    ) -> Result<usize, std::io::Error> {
        self.socket.send_to(data, addr).await
    }

    pub async fn recv_from(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(usize, SocketAddr), std::io::Error> {
        self.socket.recv_from(buf).await
    }

    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }

    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }
}
