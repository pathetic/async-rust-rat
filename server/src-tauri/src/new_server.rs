use tokio::net::TcpListener;

use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

use std::sync::{ Arc, Mutex };
use tauri::{ AppHandle, Manager };

use common::{async_impl::packets::ClientInfo, buffers::{ read_buffer, write_buffer }};

use rand::rngs::OsRng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rsa::{pkcs8::ToPublicKey, PaddingScheme, RsaPrivateKey, RsaPublicKey};

use common::async_impl::packets::*;

use anyhow::{Context, Result};

use crate::commands::*;

use common::{ENC_TOK_LEN, RSA_BITS};


pub struct ServerWrapper {
    receiver: Receiver<ServerCommand>,
    txs: HashMap<std::net::SocketAddr, Sender<ClientCommand>>,
    connected_users: HashMap<std::net::SocketAddr, ClientInfo>,
    salt_generator: ChaCha20Rng,
    priv_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
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
            salt_generator: ChaCha20Rng::from_entropy(),
            priv_key,
            pub_key,
        };

        s.channel_loop().await;

        Ok(())
    }
    
    async fn channel_loop(mut self) {
        loop {
            use crate::commands::ServerCommand::*;

            let p = match self.receiver.recv().await {
                Some(p) => p,
                None => break,
            };

            match p {
                Close => {
                    break;
                }
                Write(p) => {
                    match &p {
                        ClientboundPacket::InitClient => {
                            // to implement
                        }
                        ClientboundPacket::ScreenshotDisplay(display) => {
                            // to implement
                        }
                        ClientboundPacket::Reconnect => {
                            // to implement
                        }
                        ClientboundPacket::Disconnect => {
                            // to implement
                        }
                        _ => (),
                    }
                    // for (addr, tx_) in &self.txs {
                    //     // Only send to logged in users
                    //     // Maybe there is a prettier way to achieve that? Seems suboptimal
                    //     if self.connected_users.contains_key(addr) {
                    //         tx_.send(ConnectionCommand::Write(p.clone())).await.ok();
                    //     }
                    // }
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
                    }
                }
            }
        }
    }
}