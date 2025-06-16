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
        let mut macs = Vec::new();

        // Try to get MAC addresses using ip command
        if let Ok(output) = Command::new("ip")
            .args(["link", "show"])
            .output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("ether") && !line.contains("lo") {
                    if let Some(mac) = line.split_whitespace()
                        .find(|s| s.contains(":")) {
                        macs.push(mac.to_string());
                    }
                }
            }
        }

        // Fallback to reading from /sys/class/net
        if macs.is_empty() {
            if let Ok(entries) = fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_string_lossy();
                    
                    // Skip loopback and virtual interfaces
                    if name == "lo" || name.starts_with("docker") || name.starts_with("veth") {
                        continue;
                    }

                    if let Ok(mac) = fs::read_to_string(path.join("address")) {
                        let mac = mac.trim();
                        if !mac.is_empty() && mac != "00:00:00:00:00:00" {
                            macs.push(mac.to_string());
                        }
                    }
                }
            }
        }

        if macs.is_empty() {
            None
        } else {
            Some(macs.join(", "))
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
                    if let Some(uuid) = device.strip_prefix("UUID=") {
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
