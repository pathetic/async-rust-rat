use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
pub struct Log {
    pub event_type: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Logger {
    pub logs: Vec<Log>,
    pub tauri_handle: Option<Arc<Mutex<AppHandle>>>,
}

impl Logger {
    pub fn new() -> Self {
        Self { logs: Vec::new(), tauri_handle: None }
    }
    
    pub async fn log(&mut self, event_type: &str, message: &str) {
        let log = Log { event_type: event_type.to_string(), message: message.to_string() };
        self.logs.push(log.clone());

        if let Some(handle) = &self.tauri_handle {
            handle.lock().unwrap().emit_all("server_log", log).unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
        }
    }

    pub async fn log_once(&mut self, log: Log) {
        self.logs.push(log.clone());

        if let Some(handle) = &self.tauri_handle {
            handle.lock().unwrap().emit_all("server_log", log).unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
        }
    }
}