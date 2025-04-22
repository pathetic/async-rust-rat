use serde::{Serialize, Deserialize};
use std::sync::{ Arc, Mutex };
pub mod tauri;
use tokio::sync::mpsc::Sender;
use once_cell::sync::OnceCell;
use crate::commands::ServerCommand;

pub struct SharedTauriState(pub Arc<Mutex<TauriState>>);

pub struct TauriState {
    pub port: String,
    pub running: bool,
    channel_tx: OnceCell<Sender<ServerCommand>>,
    server_task: Option<tokio::task::JoinHandle<()>>,
    listener_task: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyInfo {
    pub assembly_name: String,
    pub assembly_description: String,
    pub assembly_company: String,
    pub assembly_copyright: String,
    pub assembly_trademarks: String,
    pub assembly_original_filename: String,
    pub assembly_product_version: String,
    pub assembly_file_version: String,
}

impl Default for TauriState {
    fn default() -> Self {
        TauriState {
            port: "1337".to_string(),
            running: false,
            channel_tx: OnceCell::new(),
            server_task: None,
            listener_task: None,
        }
    }
}