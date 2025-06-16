use common::client_info::UniqueInfo;

#[cfg(windows)]
mod imp {
    use std::process::Command;
    use std::collections::HashMap;
    use wmi::{COMLibrary, Variant, WMIConnection};
    use tokio::task;
    use common::client_info::UniqueInfo;

    pub async fn collect_unique_info() -> UniqueInfo {
        let mac_task = task::spawn_blocking(get_mac_address);
        let vol_task = task::spawn_blocking(get_volume_serial);

        let (mac, vol) = tokio::join!(mac_task, vol_task);

        UniqueInfo {
            mac_address: mac.ok().flatten().unwrap_or_else(|| "Unknown".to_string()),
            volume_serial: vol.ok().flatten().unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    fn get_volume_serial() -> Option<String> {
        let output = Command::new("cmd")
            .args(&["/C", "vol C:"])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if let Some(pos) = line.find("Serial Number is") {
                return Some(line[pos + 18..].trim().replace("-", ""));
            }
        }

        None
    }

    fn get_mac_address() -> Option<String> {
        let com = COMLibrary::new().ok()?;
        let wmi = WMIConnection::with_namespace_path("root\\cimv2", com).ok()?;

        let adapters: Vec<HashMap<String, Variant>> =
            wmi.raw_query("SELECT MACAddress FROM Win32_NetworkAdapter WHERE MACAddress IS NOT NULL").ok()?;

        adapters
            .into_iter()
            .filter_map(|mut a| {
                a.remove("MACAddress")
                    .and_then(|v| v.try_into().ok())
                    .map(|s: String| s.replace(":", ""))
            })
            .next()
    }
}

#[cfg(unix)]
mod imp {
    use std::process::Command;
    use std::fs;
    use std::path::Path;
    use tokio::task;
    use common::client_info::UniqueInfo;

    pub async fn collect_unique_info() -> UniqueInfo {
        let mac_task = task::spawn_blocking(get_mac_address);
        let vol_task = task::spawn_blocking(get_volume_serial);

        let (mac, vol) = tokio::join!(mac_task, vol_task);

        UniqueInfo {
            mac_address: mac.ok().flatten().unwrap_or_else(|| "Unknown".to_string()),
            volume_serial: vol.ok().flatten().unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    fn get_mac_address() -> Option<String> {
        // Use cat to read all MAC addresses
        if let Ok(output) = Command::new("cat")
            .args(&["/sys/class/net/*/address"])
            .output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let macs: Vec<String> = output_str
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && !s.starts_with("00:00:00:00:00:00"))
                .filter(|s| !s.contains("lo") && !s.contains("docker") && !s.contains("veth"))
                .map(|s| s.to_string())
                .collect();

            if macs.is_empty() {
                None
            } else {
                Some(macs.join(", "))
            }
        } else {
            None
        }
    }

    fn get_volume_serial() -> Option<String> {
        let mut uuids = Vec::new();

        if let Ok(fstab) = fs::read_to_string("/etc/fstab") {
            for line in fstab.lines() {
                // Skip comments and empty lines
                if line.trim_start().starts_with('#') || line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let device = parts[0];
                    let mount_point = parts[1];

                    // Skip special filesystems and system mounts
                    if mount_point.contains("/dev/") || 
                       mount_point.contains("/sys/") ||
                       mount_point.contains("/run/") ||
                       mount_point.contains("swap") {
                        continue;
                    }

                    // Extract UUID if present
                    if device.starts_with("UUID=") {
                        let uuid = &device[5..];
                        if !uuid.is_empty() {
                            uuids.push(format!("{} ({})", uuid, mount_point));
                        }
                    }
                    // Extract LUKS UUID if present
                    else if device.starts_with("/dev/mapper/luks-") {
                        let uuid = device.replace("/dev/mapper/luks-", "");
                        if !uuid.is_empty() {
                            uuids.push(format!("{} ({})", uuid, mount_point));
                        }
                    }
                }
            }
        }

        if uuids.is_empty() {
            None
        } else {
            Some(uuids.join(", "))
        }
    }
}

pub use imp::collect_unique_info;
