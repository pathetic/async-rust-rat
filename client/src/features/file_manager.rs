use std::path::PathBuf;

use common::packets::{ File, FileData, ServerboundPacket };
use crate::handler::send_packet;

pub struct FileManager {
    pub current_path: PathBuf,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::new(),
        }
    }

    pub async fn list_available_disks(&mut self) {
        let _ = send_packet(ServerboundPacket::DisksResult(get_available_disks())).await;
    }

    pub async fn write_current_folder(&mut self) {
        println!("Sending current folder: {}", self.current_path.to_string_lossy());
        let _ = send_packet(
            ServerboundPacket::CurrentFolder(self.current_path.to_string_lossy().to_string()),
        ).await;
    }

    pub async fn view_folder(
        &mut self,
        folder: &str,
    ) {
        self.current_path.push(folder);
        self.list_directory_contents().await;
        self.write_current_folder().await;
    }

    pub async fn navigate_to_parent(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.list_directory_contents().await;
        } else {
            self.list_available_disks().await;
        }
        self.write_current_folder().await;
    }

    pub async fn remove_directory(
        &mut self,
        directory: &str,
    ) {
        let dir_path = self.current_path.join(directory);
        if std::fs::remove_dir_all(dir_path).is_ok() {
            self.list_directory_contents().await;
        }
    }

    pub async fn remove_file(
        &mut self,
        file: &str,
    ) {
        let file_path = self.current_path.join(file);
        if std::fs::remove_file(file_path).is_ok() {
            self.list_directory_contents().await;
        }
    }

    pub async fn list_directory_contents(&self) {
        if let Ok(entries) = self.current_path.read_dir() {
            let mut file_entries: Vec<File> = Vec::new();
            for entry in entries.filter_map(Result::ok) {
                match entry.file_type() {
                    Ok(file_type) => {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if file_type.is_dir() {
                            file_entries.push(File {
                                file_type: "dir".to_string(),
                                name: name.clone(),
                            });
                        } else if file_type.is_file() {
                            file_entries.push(File {
                                file_type: "file".to_string(),
                                name: name.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read file type: {}", e);
                    }
                }
            }
            println!("Sending file list: {:?}", file_entries);
            let _ = send_packet(ServerboundPacket::FileList(file_entries)).await;
        } else {
            eprintln!("Could not read directory: {}", self.current_path.display());
        }
    }

    pub async fn download_file(
        &self,
        filename: &str,
    ) {
        let file_path = self.current_path.join(filename);
        if let Ok(data) = std::fs::read(&file_path) {
            let _ = send_packet(ServerboundPacket::DonwloadFileResult(FileData { name: filename.to_string(), data })).await;
        } else {
            eprintln!("Failed to read file: {}", file_path.display());
        }
    }
}

// pub fn file_manager(
//     write_stream: &mut TcpStream,
//     current_path: &mut PathBuf,
//     command: &str,
//     path: &str,
//     secret: &Option<Vec<u8>>
// ) {
//     match command {
//         "AVAILABLE_DISKS" => list_available_disks(write_stream, secret),
//         "PREVIOUS_DIR" => navigate_to_parent(write_stream, current_path, secret),
//         "VIEW_DIR" => view_folder(write_stream, current_path, path, secret),
//         "REMOVE_DIR" => remove_directory(write_stream, current_path, path, secret),
//         "REMOVE_FILE" => remove_file(write_stream, current_path, path, secret),
//         "DOWNLOAD_FILE" => download_file(write_stream, current_path, path, secret),
//         _ => {}
//     }
// }

fn get_available_disks() -> Vec<String> {
    let arr = [
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
        "L",
        "M",
        "N",
        "O",
        "P",
        "Q",
        "R",
        "S",
        "T",
        "U",
        "V",
        "W",
        "X",
        "Y",
        "Z",
    ];
    let mut available: Vec<String> = Vec::new();
    for dr in arr {
        let str = format!("{}:\\", dr);
        if std::path::Path::new(str.as_str()).read_dir().is_ok() {
            let _ = &available.push(dr.to_string());
        }
    }

    available
}