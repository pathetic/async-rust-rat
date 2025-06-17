#[cfg(windows)]
mod imp {
    use std::process::Command;
    use std::collections::HashMap;
    use wmi::{COMLibrary, Variant, WMIConnection};
    use tokio::task;
    use common::client_info::SecurityInfo;

    pub async fn collect_security_info() -> SecurityInfo {
        let firewall_future = task::spawn_blocking(|| {
            let output = Command::new("powershell")
                .args(&[
                    "-NoLogo", 
                    "-NoProfile", 
                    "-NonInteractive", 
                    "-WindowStyle", "Hidden",
                    "-Command", 
                    "Get-NetFirewallProfile | Select-Object -ExpandProperty Enabled"
                ])
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
            firewall_type: "Windows Firewall".to_string(),
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
}

#[cfg(unix)]
mod imp {
    use std::path::Path;
    use common::client_info::SecurityInfo;

    pub async fn collect_security_info() -> SecurityInfo {
        let (firewall_type, firewall_enabled) = detect_firewall_status();
        let antivirus_names = detect_antivirus();

        SecurityInfo {
            firewall_type,
            firewall_enabled,
            antivirus_names,
        }
    }

    fn detect_firewall_status() -> (String, bool) {
        // Check for UFW by looking for the binary
        if Path::new("/usr/sbin/ufw").exists() {
            return ("UFW".to_string(), true);
        }

        // Check for firewalld by looking for the binary
        if Path::new("/usr/bin/firewalld").exists() || Path::new("/usr/sbin/firewalld").exists() {
            return ("firewalld".to_string(), true);
        }

        // Check for iptables by looking for the binary
        if Path::new("/usr/sbin/iptables").exists() {
            return ("iptables".to_string(), true);
        }

        // No firewall detected
        ("None".to_string(), false)
    }

    fn detect_antivirus() -> Vec<String> {
        let mut antivirus = Vec::new();

        // Check for ClamAV by looking for the binary
        if Path::new("/usr/sbin/clamd").exists() {
            antivirus.push("ClamAV".to_string());
        }

        // Check for rkhunter by looking for the binary
        if Path::new("/usr/bin/rkhunter").exists() {
            antivirus.push("rkhunter".to_string());
        }

        // Check for chkrootkit by looking for the binary
        if Path::new("/usr/sbin/chkrootkit").exists() {
            antivirus.push("chkrootkit".to_string());
        }

        if antivirus.is_empty() {
            antivirus.push("No antivirus detected".to_string());
        }

        antivirus
    }
}

pub use imp::collect_security_info;
