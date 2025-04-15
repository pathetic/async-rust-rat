use serde::Serialize;
use std::sync::{ Arc, Mutex };
pub mod tauri;
use tokio::sync::mpsc::Sender;
use once_cell::sync::OnceCell;
use crate::commands::ServerCommand;

pub struct SharedTauriState(pub Arc<Mutex<TauriState>>);

#[derive(Debug, Clone, Serialize)]
pub struct FrontClient {
    pub addr: std::net::SocketAddr,
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub ram: String,
    pub cpu: String,
    pub gpus: Vec<String>,
    pub storage: Vec<String>,
    pub displays: i32,
    pub ip: String,
    pub disconnected: bool,
    pub is_elevated: bool,
}

pub struct TauriState {
    pub port: String,
    pub running: bool,
    channel_tx: OnceCell<Sender<ServerCommand>>,
}

impl Default for TauriState {
    fn default() -> Self {
        TauriState {
            port: "1337".to_string(),
            running: false,
            channel_tx: OnceCell::new(),
        }
    }
}
