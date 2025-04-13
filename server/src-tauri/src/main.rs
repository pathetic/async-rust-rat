#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{ Arc, Mutex };
use tauri::Manager;

mod client;
mod server;
mod handlers;

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
    SharedServer,
    TauriState,
};

#[tokio::main(worker_threads = 3)]
async fn main() {
    tauri::Builder
        ::default()
        .manage(SharedTauriState(Arc::new(Mutex::new(TauriState::default()))))
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let shared_server = SharedServer(Arc::new(Mutex::new(server::Server::new(app_handle))));

            app.manage(shared_server);

            Ok(())
        })
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
