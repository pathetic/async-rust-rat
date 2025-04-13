use std::net::TcpStream;
use std::sync::{ Arc, Mutex };

use common::buffers::read_buffer;
use rand::{ rngs::OsRng, Rng };
use std::process;
use crate::{ SECRET, SECRET_INITIALIZED };

use common::commands::Command;

use crate::features::encryption::generate_secret;
use crate::features::other::{client_info, take_screenshot};

pub fn handle_server(
    mut read_stream: TcpStream,
    mut write_stream: TcpStream,
    is_connected: Arc<Mutex<bool>>,
    is_connecting: Arc<Mutex<bool>>
) {
    OsRng.fill(&mut *SECRET.lock().unwrap());

    loop {
        let secret_clone = Some(SECRET.lock().unwrap().to_vec());
        let received_command = read_buffer(&mut read_stream, if
            SECRET_INITIALIZED.lock().unwrap().clone()
        {
            &secret_clone
        } else {
            &None
        });

        match received_command {
            Ok(command) => {
                //println!("Received command: {:?}", command);
                match command {
                    Command::EncryptionRequest(data) => {
                        generate_secret(&mut write_stream, data);
                    }
                    Command::InitClient => {
                        client_info(&mut write_stream, &Some(SECRET.lock().unwrap().to_vec()));
                    }
                    Command::Reconnect => {
                        *crate::SECRET_INITIALIZED.lock().unwrap() = false;
                        *is_connected.lock().unwrap() = false;
                        *is_connecting.lock().unwrap() = false;
                        break;
                    }
                    Command::Disconnect => {
                        process::exit(1);
                    }
                    Command::ScreenshotDisplay(data) => {
                        take_screenshot(
                            &mut write_stream,
                            data.parse::<i32>().unwrap(),
                            &Some(SECRET.lock().unwrap().to_vec())
                        );
                    }
                    _ => {
                        println!("Received an unknown or unhandled command.");
                    }
                }
            }
            Err(_) => {
                println!("Disconnected!");
                {
                    *crate::SECRET_INITIALIZED.lock().unwrap() = false;
                    *is_connected.lock().unwrap() = false;
                    *is_connecting.lock().unwrap() = false;
                }
                break;
            }
        }
    }
}
