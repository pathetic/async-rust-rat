#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{ Arc, Mutex };

mod handlers;
mod server;
mod commands;
mod client;

use handlers::{
    tauri::{
        build_client,
        fetch_client,
        fetch_state,
        fetch_clients,
        start_server,
        take_screenshot,
        manage_client
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
                take_screenshot,
                manage_client
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
