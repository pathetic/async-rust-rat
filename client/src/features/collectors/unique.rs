// src/collectors/unique.rs
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
