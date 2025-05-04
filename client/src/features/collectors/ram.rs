// src/collectors/ram.rs
use sysinfo::System;
use tokio::task;
use common::client_info::RamInfo;

pub async fn collect_ram_info() -> RamInfo {
    task::spawn_blocking(|| {
        let mut system = System::new_all();
        system.refresh_memory();

        let total_bytes = system.total_memory();
        let used_bytes = system.used_memory();

        RamInfo {
            total_gb: total_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
            used_gb: used_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
        }
    }).await.unwrap_or(RamInfo {
        total_gb: 0.0,
        used_gb: 0.0,
    })
}