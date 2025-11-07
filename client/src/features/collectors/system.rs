#[cfg(windows)]
mod windows {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    use std::process::Command;
    use tokio::task;
    use sysinfo::System;
    use common::client_info::SystemInfo;
    use crate::service::install::is_elevated;

    pub async fn collect_system_info() -> SystemInfo {
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "__UNKNOWN__".to_string());
        let machine_name = System::host_name().unwrap_or_else(|| "__UNKNOWN__".to_string());

        let reg_task = task::spawn_blocking(|| {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
            match key {
                Ok(key) => {
                    let serial = key.get_value("ProductId").ok();
                    let name = key.get_value("ProductName").ok();
                    (serial, name)
                }
                Err(_) => (None, None),
            }
        });

        let model_task = task::spawn_blocking(|| {
            let output = Command::new("powershell")
                .args(&[
                    "-NoLogo", 
                    "-NoProfile", 
                    "-NonInteractive", 
                    "-WindowStyle", "Hidden", 
                    "-Command", 
                    "Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty Model; Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty Manufacturer"
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let lines: Vec<_> = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|l| l.trim().to_string())
                        .collect();
                    let model = lines.get(0).cloned();
                    let manufacturer = lines.get(1).cloned();
                    (model, manufacturer)
                }
                _ => (None, None),
            }
        });

        let (reg_info, model_info) = tokio::join!(reg_task, model_task);

        let reg_info = reg_info.ok().unwrap_or((None, None));
        let model_info = model_info.ok().unwrap_or((None, None));
        let os_version = System::os_version();

        SystemInfo {
            username,
            machine_name,
            system_model: model_info.0,
            system_manufacturer: model_info.1,
            os_full_name: reg_info.1,
            os_version,
            os_serial_number: reg_info.0,
            is_elevated: is_elevated(),
        }
    }
}

#[cfg(unix)]
mod unix {
    use sysinfo::System;
    use common::client_info::SystemInfo;
    use std::process::Command;
    use std::fs;

    pub async fn collect_system_info() -> SystemInfo {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        // Try multiple methods to get OS information
        let (os_name, os_version) = {
            // First try /etc/os-release
            if let Ok(os_info) = fs::read_to_string("/etc/os-release") {
                let name = os_info
                    .lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .and_then(|line| line.split_once('='))
                    .map(|(_, value)| value.trim_matches('"').to_string())
                    .unwrap_or_else(|| "Linux".to_string());

                let version = os_info
                    .lines()
                    .find(|line| line.starts_with("VERSION_ID="))
                    .and_then(|line| line.split_once('='))
                    .map(|(_, value)| value.trim_matches('"').to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                (name, version)
            } else {
                // Try /etc/lsb-release as fallback
                if let Ok(lsb_info) = fs::read_to_string("/etc/lsb-release") {
                    let name = lsb_info
                        .lines()
                        .find(|line| line.starts_with("DISTRIB_DESCRIPTION="))
                        .and_then(|line| line.split_once('='))
                        .map(|(_, value)| value.trim_matches('"').to_string())
                        .unwrap_or_else(|| "Linux".to_string());

                    let version = lsb_info
                        .lines()
                        .find(|line| line.starts_with("DISTRIB_RELEASE="))
                        .and_then(|line| line.split_once('='))
                        .map(|(_, value)| value.trim_matches('"').to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    (name, version)
                } else {
                    // If all else fails, try to get basic Linux info
                    let uname = Command::new("uname")
                        .args(["-a"])
                        .output()
                        .ok()
                        .and_then(|output| String::from_utf8(output.stdout).ok())
                        .unwrap_or_else(|| "Linux".to_string());

                    (uname, "Unknown".to_string())
                }
            }
        };

        // Get system model from /sys/devices/virtual/dmi/id/product_name
        let system_model = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .unwrap_or_else(|_| "Linux PC".to_string())
            .trim()
            .to_string();

        // Get manufacturer from /sys/devices/virtual/dmi/id/sys_vendor
        let system_manufacturer = fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        // Get system serial number from /sys/devices/virtual/dmi/id/product_serial
        let os_serial_number = fs::read_to_string("/sys/devices/virtual/dmi/id/product_serial")
            .ok()
            .map(|s| s.trim().to_string());

        SystemInfo {
            username: std::env::var("USER").unwrap_or_else(|_| "__UNKNOWN__".to_string()),
            machine_name: System::host_name().unwrap_or_else(|| "__UNKNOWN__".to_string()),
            system_model: Some(system_model),
            system_manufacturer: Some(system_manufacturer),
            os_full_name: Some(os_name),
            os_version: Some(os_version),
            os_serial_number,
            is_elevated: unsafe { libc::geteuid() == 0 },
        }
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;
