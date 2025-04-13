use serde::Serialize;
use crate::server::Server;
use std::sync::{ Arc, Mutex };

pub mod tauri;

pub struct SharedServer(pub Arc<Mutex<Server>>);
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

#[derive(Debug, Clone, Serialize)]
pub struct TauriState {
    pub port: String,
    pub running: bool,
}

impl Default for TauriState {
    fn default() -> Self {
        TauriState {
            port: "1337".to_string(),
            running: false,
        }
    }
}
