// src/collectors/drives.rs
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};
use tokio::task;
use common::client_info::PhysicalDrive;

pub async fn collect_physical_drives() -> Vec<PhysicalDrive> {
    task::spawn_blocking(|| {
        let com = match COMLibrary::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ COM init failed: {:?}", e);
                return Vec::new();
            }
        };

        let wmi = match WMIConnection::with_namespace_path("root\\cimv2", com) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ WMI connection failed: {:?}", e);
                return Vec::new();
            }
        };

        let results: Vec<HashMap<String, Variant>> = match wmi.raw_query("SELECT Model, Size FROM Win32_DiskDrive") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("❌ WMI diskdrive query failed: {:?}", e);
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