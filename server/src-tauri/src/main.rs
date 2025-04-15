#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{ Arc, Mutex };
use tauri::Manager;

// mod client;
// mod server;
mod handlers;
mod new_server;
mod commands;
mod new_client;

use handlers::{
    tauri::{
        build_client,
        fetch_client,
        fetch_state,
        fetch_clients,
        start_server,
        take_screenshot
    },
    SharedTauriState,
    TauriState,
};

#[tokio::main(worker_threads = 3)]
async fn main() {
    tauri::Builder
        ::default()
        .manage(SharedTauriState(Arc::new(Mutex::new(TauriState::default()))))
        .invoke_handler(
            tauri::generate_handler![
                start_server,
                fetch_state,
                build_client,
                fetch_clients,
                fetch_client,
                take_screenshot
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
