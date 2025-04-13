use std::net::{ TcpListener, TcpStream };
use std::sync::{ Arc, Mutex };
use tauri::{ AppHandle, Manager };

use crate::client;
use common::buffers::{ read_buffer, write_buffer };

use rand::rngs::OsRng;
use rsa::{ RsaPrivateKey, RsaPublicKey };
use anyhow::Context;
use rsa::pkcs8::EncodePublicKey;
use rsa::Pkcs1v15Encrypt;

use common::commands::{ ClientInfo, Command, EncryptionRequestData, EncryptionResponseData };

fn encryption_request(mut stream: TcpStream, pub_key: Vec<u8>) -> EncryptionResponseData {
    write_buffer(
        &mut stream,
        Command::EncryptionRequest(EncryptionRequestData { public_key: pub_key.clone() }),
        &None
    );

    let rcv = read_buffer(&mut stream.try_clone().unwrap(), &None).unwrap();

    match rcv {
        Command::EncryptionResponse(response) => response,
        _ => panic!("Invalid command received"),
    }
}

fn get_client(mut stream: TcpStream, secret: Option<Vec<u8>>) -> ClientInfo {
    write_buffer(&mut stream, Command::InitClient, &secret);

    let rcv = read_buffer(&mut stream.try_clone().unwrap(), &secret).unwrap();

    match rcv {
        Command::Client(client) => client,
        _ => panic!("Invalid command received"),
    }
}

#[derive(Clone)]
pub struct Server {
    pub tauri_handle: Arc<Mutex<AppHandle>>,
    pub clients: Arc<Mutex<Vec<client::Client>>>,

    private_key: rsa::RsaPrivateKey,
    public_key: rsa::RsaPublicKey,
}

impl Server {
    pub fn new(app_handle: AppHandle) -> Self {
        let mut rng = OsRng;

        let priv_key = RsaPrivateKey::new(&mut rng, common::RSA_BITS)
            .with_context(|| "Failed to generate a key.")
            .unwrap();
        let pub_key = RsaPublicKey::from(&priv_key);

        Server {
            tauri_handle: Arc::new(Mutex::new(app_handle)),
            clients: Arc::new(Mutex::new(Vec::new())),
            private_key: priv_key,
            public_key: pub_key,
        }
    }

    pub fn listen_port(&mut self, port: String) -> bool {
        let listener = TcpListener::bind("0.0.0.0:".to_string() + port.as_str());

        if let Ok(stream) = listener {
            let vec = Arc::clone(&self.clients);
            let tauri_handle = Arc::clone(&self.tauri_handle);
            let public_key = self.public_key.clone();
            let private_key = self.private_key.clone();

            tokio::task::spawn(async move {
                for i in stream.incoming() {
                    println!("New Client!");

                    if i.is_err() {
                        break;
                    }

                    let stream = i.unwrap();
                    let ip = stream.try_clone().unwrap().peer_addr().unwrap().ip().to_string();

                    let encryption = encryption_request(
                        stream.try_clone().unwrap(),
                        public_key.to_public_key_der().unwrap().as_ref().to_vec()
                    );

                    let padding = Pkcs1v15Encrypt::default();
                    let secret_dec = private_key.decrypt(padding, &encryption.secret).unwrap();

                    let info = get_client(stream.try_clone().unwrap(), Some(secret_dec.clone()));

                    let mut client = client::Client::new(
                        Arc::clone(&tauri_handle),
                        stream.try_clone().unwrap(),
                        secret_dec,
                        info.username.clone(),
                        info.hostname,
                        info.os,
                        info.ram,
                        info.cpu,
                        info.gpus,
                        info.storage,
                        info.displays,
                        ip,
                        info.is_elevated
                    );

                    client.is_handled = true;
                    client.handle_client();

                    let _ = tauri_handle
                        .lock()
                        .unwrap()
                        .emit_all("client_connected", info.username);

                    vec.lock().unwrap().push(client);
                }
            });
            true
        } else {
            false
        }
    }
}
