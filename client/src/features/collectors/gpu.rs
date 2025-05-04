// src/collectors/gpu.rs
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};
use tokio::task;
use common::client_info::GpuInfo;

pub async fn collect_gpu_info() -> Vec<GpuInfo> {
    task::spawn_blocking(|| {
        let com = match COMLibrary::new() {
            Ok(c) => c,
            Err(_e) => {
                return Vec::new();
            }
        };

        let wmi = match WMIConnection::with_namespace_path("root\\cimv2", com) {
            Ok(w) => w,
            Err(_e) => {
                return Vec::new();
            }
        };

        let results: Vec<HashMap<String, Variant>> = match wmi.raw_query("SELECT Name, DriverVersion FROM Win32_VideoController") {
            Ok(r) => r,
            Err(_e) => {
                return Vec::new();
            }
        };

        results
            .into_iter()
            .map(|mut gpu| {
                let name = gpu
                    .remove("Name")
                    .and_then(|v| v.try_into().ok())
                    .unwrap_or_else(|| "Unknown".to_string());

                let driver_version = gpu
                    .remove("DriverVersion")
                    .and_then(|v| v.try_into().ok());

                GpuInfo {
                    name,
                    driver_version,
                }
            })
            .collect()
    }).await.unwrap_or_default()
}