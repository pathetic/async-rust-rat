use std::net::TcpStream;
use std::sync::{ Arc, Mutex };
use base64::{ engine::general_purpose, Engine as _ };
use tauri::{ AppHandle, Manager };

use common::buffers::{ read_buffer, write_buffer };
use common::commands::{ Command };

#[derive(Debug)]
pub struct Client {
    tauri_handle: Arc<Mutex<AppHandle>>,
    write_stream: TcpStream,
    read_stream: Arc<Mutex<TcpStream>>,
    secret: Vec<u8>,

    username: String,
    hostname: String,
    os: String,
    ram: String,
    cpu: String,
    gpus: Vec<String>,
    storage: Vec<String>,
    displays: i32,
    ip: String,

    is_elevated: bool,

    disconnected: Arc<Mutex<bool>>,

    pub is_handled: bool,
}

impl Client {
    pub fn new(
        tauri_handle: Arc<Mutex<AppHandle>>,
        write_stream: TcpStream,
        secret: Vec<u8>,
        username: String,
        hostname: String,
        os: String,
        ram: String,
        cpu: String,
        gpus: Vec<String>,
        storage: Vec<String>,
        displays: i32,
        ip: String,
        is_elevated: bool
    ) -> Self {
        Client {
            tauri_handle,
            write_stream: write_stream.try_clone().unwrap(),
            read_stream: Arc::new(Mutex::new(write_stream.try_clone().unwrap())),
            secret,
            username,
            hostname,
            os,
            ram,
            cpu,
            gpus,
            storage,
            displays,
            ip,
            is_elevated,
            disconnected: Arc::new(Mutex::new(false)),
            is_handled: false,
        }
    }

    pub fn write_buffer(&mut self, command: Command, secret: &Option<Vec<u8>>) {
        write_buffer(&mut self.write_stream, command, &secret);
    }

    pub fn handle_client(&mut self) {
        let username_clone = self.username.clone();
        let stream_clone = Arc::clone(&self.read_stream);
        let disconnected = Arc::clone(&self.disconnected);
        let tauri_handle = Arc::clone(&self.tauri_handle);
        let secret = self.get_secret();
        
        std::thread::spawn(move || {
            loop {
                let mut locked_stream = stream_clone.lock().unwrap();
                let received_data = read_buffer(&mut locked_stream, &Some(secret.clone()));

                match received_data {
                    Ok(received) => {
                        match received {
                            Command::ScreenshotResult(screenshot) => {
                                let base64_img = general_purpose::STANDARD.encode(screenshot);
                                let _ = tauri_handle
                                    .lock()
                                    .unwrap()
                                    .emit_all("client_screenshot", base64_img);
                            }
                            _ => {
                                println!("Received unknown or unhandled data.");
                            }
                        }
                    }
                    Err(_) => {
                        let _ = tauri_handle
                            .lock()
                            .unwrap()
                            .emit_all("client_disconnected", username_clone);
                        println!("Disconnected!");
                        break;
                    }
                }
            }
            *disconnected.lock().unwrap() = true;
        });
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_hostname(&self) -> String {
        self.hostname.clone()
    }

    pub fn get_os(&self) -> String {
        self.os.clone()
    }

    pub fn get_ram(&self) -> String {
        self.ram.clone()
    }

    pub fn get_cpu(&self) -> String {
        self.cpu.clone()
    }

    pub fn get_gpus(&self) -> Vec<String> {
        self.gpus.clone()
    }

    pub fn get_storage(&self) -> Vec<String> {
        self.storage.clone()
    }

    pub fn get_displays(&self) -> i32 {
        self.displays
    }

    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }

    pub fn is_elevated(&self) -> bool {
        self.is_elevated
    }

    pub fn is_disconnect(&self) -> bool {
        *self.disconnected.lock().unwrap()
    }

    pub fn get_secret(&self) -> Vec<u8> {
        self.secret.clone()
    }
}
