#![crate_type = "cdylib"]

use std::path::PathBuf;
use std::sync::Mutex;

use common::packets::{File, FileData, ServerboundPacket};
use lazy_static::lazy_static;
use common::packets::Packet;

lazy_static! {
    static ref FILE_MANAGER: Mutex<FileManager> = Mutex::new(FileManager::new());
}

pub struct FileManager {
    current_path: PathBuf,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::new(),
        }
    }

    pub fn list_available_disks(&self) -> ServerboundPacket {
        let disks = get_available_disks();
        ServerboundPacket::DisksResult(disks)
    }

    pub fn write_current_folder(&self) -> ServerboundPacket {
        ServerboundPacket::CurrentFolder(self.current_path.to_string_lossy().to_string())
    }

    pub fn view_folder(&mut self, folder: &str) -> ServerboundPacket {
        self.current_path.push(folder);
        self.list_directory_contents()
    }

    pub fn previous_dir(&mut self) -> ServerboundPacket {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.list_directory_contents()
        } else {
            self.list_available_disks()
        }
    }


    pub fn remove_directory(&mut self, directory: &str) -> ServerboundPacket {
        let dir_path = self.current_path.join(directory);
        if std::fs::remove_dir_all(&dir_path).is_ok() {
            self.list_directory_contents()
        } else {
            ServerboundPacket::FileList(Vec::new()) // fallback
        }
    }

    pub fn remove_file(&mut self, file: &str) -> ServerboundPacket {
        let file_path = self.current_path.join(file);
        if std::fs::remove_file(&file_path).is_ok() {
            self.list_directory_contents()
        } else {
            ServerboundPacket::FileList(Vec::new()) // fallback
        }
    }

    pub fn list_directory_contents(&self) -> ServerboundPacket {
        let mut files = Vec::new();
        if let Ok(entries) = self.current_path.read_dir() {
            for entry in entries.filter_map(Result::ok) {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(file_type) = entry.file_type() {
                    files.push(File {
                        file_type: if file_type.is_dir() { "dir" } else { "file" }.to_string(),
                        name,
                    });
                }
            }
        }
        ServerboundPacket::FileList(files)
    }

    pub fn download_file(&self, filename: &str) -> ServerboundPacket {
        let file_path = self.current_path.join(filename);
        if let Ok(data) = std::fs::read(&file_path) {
            ServerboundPacket::DonwloadFileResult(FileData {
                name: filename.to_string(),
                data,
            })
        } else {
            ServerboundPacket::FileList(Vec::new()) // fallback
        }
    }
}

fn get_available_disks() -> Vec<String> {
    let arr = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
        "T", "U", "V", "W", "X", "Y", "Z",
    ];
    arr.iter()
        .filter_map(|&d| {
            let path = format!("{}:\\", d);
            if std::path::Path::new(&path).read_dir().is_ok() {
                Some(d.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[no_mangle]
pub extern "C" fn plugin_init() {
    println!("🧠 Plugin initialized!");
    let _unused = FILE_MANAGER.lock().unwrap();
}

#[no_mangle]
pub extern "C" fn plugin_execute(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut *const u8,
    output_len: *mut usize,
) -> i32 {
    unsafe {
        let input_slice = std::slice::from_raw_parts(input_ptr, input_len);
        if let Ok(command) = std::str::from_utf8(input_slice) {
            let mut manager = FILE_MANAGER.lock().unwrap();
            let packet = match command {
                "list_available_disks" => manager.list_available_disks(),
                "list_directory_contents" => manager.list_directory_contents(),
                "write_current_folder" => manager.write_current_folder(),
                "previous_dir" => manager.previous_dir(),

                cmd if cmd.starts_with("view_folder:") => {
                    let folder = &cmd["view_folder:".len()..];
                    manager.view_folder(folder)
                }
                cmd if cmd.starts_with("remove_dir:") => {
                    let folder = &cmd["remove_dir:".len()..];
                    manager.remove_directory(folder)
                }
                cmd if cmd.starts_with("remove_file:") => {
                    let file = &cmd["remove_file:".len()..];
                    manager.remove_file(file)
                }
                cmd if cmd.starts_with("download_file:") => {
                    let file = &cmd["download_file:".len()..];
                    manager.download_file(file)
                }

                _ => return -1,
            };

            let serialized = packet.serialized();
            let leaked = Box::leak(serialized.into_boxed_slice());
            *output_ptr = leaked.as_ptr();
            *output_len = leaked.len();

            0
        } else {
            -2
        }
    }
}
