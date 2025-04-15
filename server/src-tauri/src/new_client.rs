use crate::commands::*;
use common::async_impl::connection::*;
use common::async_impl::packets::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use common::async_impl::packets::{ClientInfo, ClientboundPacket};

pub struct ClientWrapper; // Maybe this shouldn't be a struct?

impl ClientWrapper {
    /// Handles incoming connection and spawns reading and writing loops.
    pub async fn spawn(
        socket: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
        ctx: Sender<ServerCommand>,
    ) {
        let (tx, rx) = mpsc::channel::<ClientCommand>(32);
        println!("Connection from: {:?}", addr);
        let connection = Connection::<ServerboundPacket, ClientboundPacket>::new(socket);
        let (reader, writer) = connection.split();
        let reader_wrapped = ClientReaderWrapper::new(reader, addr, tx, ctx);
        tokio::spawn(reader_wrapped.spawn_loop());
        let writer_wrapped = ConnectionWriterWrapper::new(writer, rx);
        tokio::spawn(writer_wrapped.spawn_loop());
    }
}

pub struct ClientReaderWrapper {
    reader: ConnectionReader<ServerboundPacket>,
    addr: std::net::SocketAddr,
    client_sender: Sender<ClientCommand>,
    server_sender: Sender<ServerCommand>,
    secret: Option<Vec<u8>>,
    nonce_generator: Option<ChaCha20Rng>,
}

impl ClientReaderWrapper {
    fn new(
        reader: ConnectionReader<ServerboundPacket>,
        addr: std::net::SocketAddr,
        client_sender: Sender<ClientCommand>,
        server_sender: Sender<ServerCommand>,
    ) -> Self {
        Self {
            reader,
            addr,
            client_sender,
            server_sender,
            secret: None,
            nonce_generator: None,
        }
    }

    async fn handle_encryption_request(&mut self) {
        use ServerboundPacket::*;
        // To send back the token
        let (otx, orx) = oneshot::channel();
        self.server_sender
            .send(ServerCommand::EncryptionRequest(
                self.client_sender.clone(),
                otx,
            ))
            .await
            .unwrap();

        let expect_token = orx.await.unwrap();

        // Now we expect EncryptionConfirm with encrypted secret and token
        match self
            .reader
            .read_packet(&self.secret, self.nonce_generator.as_mut())
            .await
        {
            Ok(Some(EncryptionConfirm(s, t))) => {
                let (otx, orx) = oneshot::channel();
                self.server_sender
                    .send(ServerCommand::EncryptionConfirm(
                        self.client_sender.clone(),
                        otx,
                        s.clone(),
                        t,
                        expect_token,
                    ))
                    .await
                    .unwrap();

                // Get decrypted secret back from channel
                match orx.await.unwrap() {
                    Ok(s) => {
                        self.secret = Some(s.clone());
                        let mut seed = [0u8; common::SECRET_LEN];
                        seed.copy_from_slice(&s);

                        self.nonce_generator = Some(ChaCha20Rng::from_seed(seed));
                    }
                    Err(_) => {
                        self.client_sender
                            .send(ClientCommand::Close)
                            .await
                            .ok(); // it's ok if already closed
                    }
                }
            }
            Ok(_) => {
                println!("Client sent wrong packet during encryption handshake.");
                self.client_sender
                    .send(ClientCommand::Close)
                    .await
                    .ok(); // it's ok if already closed
            }
            Err(_) => {
                println!("Error during encryption handshake.");
                self.client_sender
                    .send(ClientCommand::Close)
                    .await
                    .ok(); // it's ok if already closed
            }
        };
    }

    async fn handle_packet(&mut self, packet: ServerboundPacket) {
        use ServerboundPacket::*;
        match packet {
            // Users requests encryption
            EncryptionRequest => self.handle_encryption_request().await,
            
            // rest is only for logged in users
            // p => {
            //     if self.username.is_some() {
            //         match p {
            //             // User wants to send a message
            //             Message(m) => {
            //                 if verify_message(&m) {
            //                     let p = ClientboundPacket::Message(accord::packets::Message {
            //                         sender_id: self.user_id.unwrap(),
            //                         sender: self.username.clone().unwrap(),
            //                         text: m,
            //                         time: current_time_as_sec(),
            //                     });
            //                     self.channel_sender
            //                         .send(ChannelCommand::Write(p))
            //                         .await
            //                         .unwrap();
            //                 } else {
            //                     log::info!("Invalid message from {:?}: {}", self.username, m);
            //                 }
            //             }
            //             // User sends an image
            //             ImageMessage(im) => {
            //                 let p =
            //                     ClientboundPacket::ImageMessage(accord::packets::ImageMessage {
            //                         image_bytes: im,
            //                         sender_id: self.user_id.unwrap(),
            //                         sender: self.username.clone().unwrap(),
            //                         time: current_time_as_sec(),
            //                     });
            //                 self.channel_sender
            //                     .send(ChannelCommand::Write(p))
            //                     .await
            //                     .unwrap();
            //             }
            //             p => {
            //                 unreachable!("{:?} should have been handled!", p);
            //             }
            //         }
            //     } else {
            //         log::warn!("Someone tried to do something without being logged in");
            //     }
            // }
        };
    }
}