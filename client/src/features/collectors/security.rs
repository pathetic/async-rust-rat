// src/collectors/security.rs
use std::process::Command;
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};
use tokio::task;
use common::client_info::SecurityInfo;

pub async fn collect_security_info() -> SecurityInfo {
    let firewall_future = task::spawn_blocking(|| {
        let output = Command::new("powershell")
            .args(&["-Command", "Get-NetFirewallProfile | Select-Object -ExpandProperty Enabled"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .any(|line| line.trim() == "True" || line.trim() == "1")
            }
            _ => false,
        }
    });

    let av_future = task::spawn_blocking(|| get_installed_avs());

    let (firewall_enabled, antivirus_names) = tokio::join!(firewall_future, av_future);

    SecurityInfo {
        firewall_enabled: firewall_enabled.unwrap_or(false),
        antivirus_names: antivirus_names.unwrap_or_default(),
    }
}

fn get_installed_avs() -> Vec<String> {
    let com = match COMLibrary::new() {
        Ok(c) => c,
        Err(_e) => {
            return Vec::new();
        }
    };

    let wmi = match WMIConnection::with_namespace_path("root\\SecurityCenter2", com) {
        Ok(w) => w,
        Err(_e) => {
            return Vec::new();
        }
    };

    let results: Vec<HashMap<String, Variant>> = match wmi.raw_query("SELECT displayName FROM AntivirusProduct") {
        Ok(r) => r,
        Err(_e) => {
            return Vec::new();
        }
    };

    results
        .into_iter()
        .filter_map(|mut av| {
            av.remove("displayName")
                .and_then(|v| v.try_into().ok())
        })
        .collect()
}
