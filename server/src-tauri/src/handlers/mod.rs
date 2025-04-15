use serde::Serialize;
use std::sync::{ Arc, Mutex };
pub mod tauri;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::task::JoinHandle;
use once_cell::sync::OnceCell;
use crate::commands::{ServerCommand, ClientCommand};

pub struct SharedTauriState(pub Arc<Mutex<TauriState>>);

#[derive(Debug, Clone, Serialize)]
pub struct FrontClient {
    pub id: usize,
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

#[derive(Debug, Clone)]
pub struct TauriState {
    channel_rx: OnceCell<Arc<Mutex<Receiver<ServerCommand>>>>,
    channel_tx: OnceCell<Sender<ServerCommand>>,
    pub port: String,
    pub running: bool,
}

impl Default for TauriState {
    fn default() -> Self {
        TauriState {
            port: "1337".to_string(),
            running: false,
            channel_tx: OnceCell::new(),
            channel_rx: OnceCell::new(),
        }
    }
}
