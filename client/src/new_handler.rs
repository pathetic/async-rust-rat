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
                send_packet(ServerboundPacket::ClientInfo(client_info)).await.unwrap()
            }

            Ok(Some(ClientboundPacket::ScreenshotDisplay(display))) => {
                let screenshot_data = take_screenshot(display.parse::<i32>().unwrap());
                send_packet(ServerboundPacket::ScreenshotResult(screenshot_data)).await.unwrap()
            }

            Ok(Some(p)) => {
                println!("!!Unhandled packet: {:?}", p);
            }
            Err(e) => {
                println!("{}", e);
                close_sender.send(()).unwrap();
                break 'l;
            }
            _ => {
                println!("Connection closed(?)\nPress Enter to exit.");
                close_sender.send(()).unwrap();
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
    // Create a channel for sending packets to the writer
    let (packet_tx, mut packet_rx) = tokio::sync::mpsc::channel::<ServerboundPacket>(32);
    
    // Store packet_tx in a globally accessible place
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

// Create a once_cell to store the sender
static PACKET_SENDER: once_cell::sync::OnceCell<tokio::sync::mpsc::Sender<ServerboundPacket>> = once_cell::sync::OnceCell::new();

// Helper function to send packets
pub async fn send_packet(packet: ServerboundPacket) -> Result<(), String> {
    if let Some(sender) = PACKET_SENDER.get() {
        sender.send(packet).await.map_err(|e| e.to_string())
    } else {
        Err("Packet sender not initialized".to_string())
    }
}