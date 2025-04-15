use crate::features::other::{take_screenshot, client_info};

use common::async_impl::packets::*;
use rand_chacha::ChaCha20Rng;
use tokio::sync::oneshot;

use common::async_impl::connection::{ConnectionReader, ConnectionWriter};

pub async fn reading_loop(
    mut reader: ConnectionReader<ClientboundPacket>,
    close_sender: oneshot::Sender<()>,
    secret: Option<Vec<u8>>,
    mut nonce_generator: Option<ChaCha20Rng>,
) {
    'l: loop {
        match reader.read_packet(&secret, nonce_generator.as_mut()).await {
            Ok(Some(ClientboundPacket::InitClient)) => {
                println!("InitClient");
                let client_info = client_info();
                match send_packet(ServerboundPacket::ClientInfo(client_info.clone())).await {
                    Ok(_) => println!("Sent client info to server"),
                    Err(e) => {
                        println!("Error sending client info: {}", e);
                        // Wait a moment and retry once
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        match send_packet(ServerboundPacket::ClientInfo(client_info)).await {
                            Ok(_) => println!("Sent client info to server on retry"),
                            Err(e) => println!("Failed to send client info on retry: {}", e),
                        }
                    }
                }
            }

            Ok(Some(ClientboundPacket::ScreenshotDisplay(display))) => {
                println!("Taking screenshot of display: {}", display);
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
                println!("Server requested disconnect");
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }

            Ok(Some(ClientboundPacket::Reconnect)) => {
                println!("Server requested reconnect - exiting to allow client to restart");
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }

            Ok(Some(p)) => {
                println!("!!Unhandled packet: {:?}", p);
            }
            
            Err(e) => {
                println!("{}", e);
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }
            
            _ => {
                println!("Connection closed(?)\nPress Enter to exit.");
                close_sender.send(()).unwrap_or_else(|_| println!("Failed to send close signal"));
                break 'l;
            }
        }
    }
}

pub async fn writing_loop(
    mut writer: ConnectionWriter<ServerboundPacket>,
    mut rx: oneshot::Receiver<()>,
    secret: Option<Vec<u8>>,
    mut nonce_generator: Option<ChaCha20Rng>,
) {
    let (packet_tx, mut packet_rx) = tokio::sync::mpsc::channel::<ServerboundPacket>(32);
    
    PACKET_SENDER.set(packet_tx).unwrap();
    
    loop {
        tokio::select! {
            // Check if it's time to close
            _ = &mut rx => {
                println!("Closing writing loop");
                break;
            },
            // Process packets from the channel
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

static PACKET_SENDER: once_cell::sync::OnceCell<tokio::sync::mpsc::Sender<ServerboundPacket>> = once_cell::sync::OnceCell::new();

pub async fn send_packet(packet: ServerboundPacket) -> Result<(), String> {
    for _ in 0..5 {
        if let Some(sender) = PACKET_SENDER.get() {
            return sender.send(packet).await.map_err(|e| e.to_string());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // If we get here, the sender is still not initialized after retries
    Err("Packet sender not initialized after multiple attempts".to_string())
}