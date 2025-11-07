#[cfg(windows)]
mod imp {
    use std::collections::HashMap;
    use wmi::{COMLibrary, Variant, WMIConnection};
    use tokio::task;
    use common::client_info::PhysicalDrive;

    pub async fn collect_physical_drives() -> Vec<PhysicalDrive> {
        task::spawn_blocking(|| {
            let com = match COMLibrary::new() {
                Ok(c) => c,
                Err(_e) => {
                    return Vec::new();
                }
            };

            let wmi = match WMIConnection::with_namespace_path("root\\cimv2", com) {
                Ok(c) => c,
                Err(_e) => {
                    return Vec::new();
                }
            };

            let results: Vec<HashMap<String, Variant>> = match wmi.raw_query("SELECT Model, Size FROM Win32_DiskDrive") {
                Ok(r) => r,
                Err(_e) => {
                    return Vec::new();
                }
            };

            results
                .into_iter()
                .map(|mut disk| {
                    let model = disk
                        .remove("Model")
                        .and_then(|v| v.try_into().ok())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let size_bytes: u64 = disk
                        .remove("Size")
                        .and_then(|v| v.try_into().ok())
                        .unwrap_or(0);

                    PhysicalDrive {
                        model,
                        size_gb: size_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                    }
                })
                .collect()
        }).await.unwrap_or_default()
    }
}

#[cfg(unix)]
mod imp {
    use common::client_info::PhysicalDrive;
    use tokio::task;
    use std::fs;

    pub async fn collect_physical_drives() -> Vec<PhysicalDrive> {
        task::spawn_blocking(|| {
            let mut drives = Vec::new();
            
            // Read from /sys/block directory
            if let Ok(entries) = fs::read_dir("/sys/block") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(device_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip loopback and ram devices
                        if device_name.starts_with("loop") || device_name.starts_with("ram") {
                            continue;
                        }

                        // Try to get the model name
                        let model = fs::read_to_string(path.join("device/model"))
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|_| "Unknown".to_string());

                        // Try to get the size from /sys/block/{device}/size
                        let size_bytes = fs::read_to_string(path.join("size"))
                            .ok()
                            .and_then(|s| s.trim().parse::<u64>().ok())
                            .unwrap_or(0) * 512; // Convert sectors to bytes

                        let size_gb = size_bytes as f64 / 1024.0 / 1024.0 / 1024.0;

                        drives.push(PhysicalDrive {
                            model,
                            size_gb,
                        });
                    }
                }
            }

            drives
        }).await.unwrap_or_default()
    }
}

pub use imp::collect_physical_drives;