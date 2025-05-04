use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use std::process::Command;
use tokio::task;
use sysinfo::System;
use common::client_info::SystemInfo;
use crate::service::install::is_elevated;

pub async fn collect_system_info() -> SystemInfo {
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "__UNKNOWN__".to_string());
    let machine_name = System::host_name().unwrap_or_else(|| "__UNKNOWN__".to_string());

    let reg_task = task::spawn_blocking(|| {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
        match key {
            Ok(key) => {
                let serial = key.get_value("ProductId").ok();
                let name = key.get_value("ProductName").ok();
                (serial, name)
            }
            Err(_) => (None, None),
        }
    });

    let model_task = task::spawn_blocking(|| {
        let output = Command::new("powershell")
            .args(&["-Command", "Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty Model; Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty Manufacturer"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let lines: Vec<_> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .collect();
                let model = lines.get(0).cloned();
                let manufacturer = lines.get(1).cloned();
                (model, manufacturer)
            }
            _ => (None, None),
        }
    });

    let (reg_info, model_info) = tokio::join!(reg_task, model_task);

    let reg_info = reg_info.ok().unwrap_or((None, None));
    let model_info = model_info.ok().unwrap_or((None, None));
    let os_version = System::os_version();

    SystemInfo {
        username,
        machine_name,
        system_model: model_info.0,
        system_manufacturer: model_info.1,
        os_full_name: reg_info.1,
        os_version,
        os_serial_number: reg_info.0,
        is_elevated: is_elevated(),
    }
}
