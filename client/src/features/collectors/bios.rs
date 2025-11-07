#[cfg(windows)]
mod windows {
    use std::collections::HashMap;
    use wmi::{COMLibrary, Variant, WMIConnection};
    use tokio::task;
    use common::client_info::BiosInfo;

    pub async fn collect_bios_info() -> BiosInfo {
        task::spawn_blocking(|| {
            let com = COMLibrary::new().ok().unwrap();
            let wmi = WMIConnection::with_namespace_path("root\\cimv2", com).ok().unwrap();

            let results: Vec<HashMap<String, Variant>> = wmi
                .raw_query("SELECT Manufacturer, Description, SerialNumber, SMBIOSBIOSVersion FROM Win32_BIOS")
                .ok().unwrap();

            let first = results.into_iter().next().unwrap();

            BiosInfo {
                manufacturer: first.get("Manufacturer").and_then(|v| v.clone().try_into().ok()).unwrap_or_else(|| "Unknown".to_string()),
                description: first.get("Description").and_then(|v| v.clone().try_into().ok()).unwrap_or_else(|| "Unknown".to_string()),
                serial_number: first.get("SerialNumber").and_then(|v| v.clone().try_into().ok()).unwrap_or_else(|| "Unknown".to_string()),
                version: first.get("SMBIOSBIOSVersion").and_then(|v| v.clone().try_into().ok()).unwrap_or_else(|| "Unknown".to_string()),
            }
        }).await.unwrap_or(BiosInfo {
            manufacturer: "Unknown".to_string(),
            description: "Unknown".to_string(),
            serial_number: "Unknown".to_string(),
            version: "Unknown".to_string(),
        })
    }
}

#[cfg(unix)]
mod unix {
    use std::process::Command;
    use tokio::task;
    use common::client_info::BiosInfo;

    pub async fn collect_bios_info() -> BiosInfo {
        task::spawn_blocking(|| {
            let manufacturer = Command::new("dmidecode")
                .args(["-s", "bios-vendor"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let description = Command::new("dmidecode")
                .args(["-s", "bios-version"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let serial_number = Command::new("dmidecode")
                .args(["-s", "system-serial-number"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let version = Command::new("dmidecode")
                .args(["-s", "bios-release-date"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            BiosInfo {
                manufacturer,
                description,
                serial_number,
                version,
            }
        }).await.unwrap_or(BiosInfo {
            manufacturer: "Unknown".to_string(),
            description: "Unknown".to_string(),
            serial_number: "Unknown".to_string(),
            version: "Unknown".to_string(),
        })
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;
