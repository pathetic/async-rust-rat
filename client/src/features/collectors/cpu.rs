use sysinfo::System;
use tokio::task;
use common::client_info::CpuInfo;

pub async fn collect_cpu_info() -> CpuInfo {
    let sys_info = task::spawn_blocking(|| {
        let mut system = System::new_all();
        system.refresh_cpu_all();

        let cpus = system.cpus();
        let cpu_name = cpus.first().map(|c| c.brand().to_string()).unwrap_or("Unknown".to_string());
        let logical_processors = cpus.len();
        let clock_speed_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);

        (cpu_name, logical_processors, clock_speed_mhz)
    }).await.unwrap_or(("Unknown".into(), 0, 0));

    let extra_info = task::spawn_blocking(|| {
        let output = std::process::Command::new("powershell")
            .args(&[
                "-NoLogo", 
                "-NoProfile", 
                "-NonInteractive", 
                "-WindowStyle", "Hidden",
                "-Command", 
                r#"
                $cpu = Get-CimInstance Win32_Processor
                $cpu.Manufacturer
                $cpu.Family
                $cpu.Description
            "#
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let lines: Vec<_> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .collect();
                (
                    lines.get(0).cloned(),
                    lines.get(1).cloned(),
                    lines.get(2).cloned(),
                )
            }
            _ => (None, None, None),
        }
    }).await.unwrap_or((None, None, None));

    CpuInfo {
        cpu_name: sys_info.0,
        logical_processors: sys_info.1,
        clock_speed_mhz: sys_info.2,
        processor_family: extra_info.1,
        manufacturer: extra_info.0,
        description: extra_info.2,
    }
}
