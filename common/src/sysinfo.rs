use std::fs;
use std::io::{self, BufRead};
use std::process::Command;
use crate::client_info::{CpuInfo, GpuInfo};

pub fn get_cpu_info() -> io::Result<CpuInfo> {
    let mut cpu_name = String::new();
    let mut manufacturer = None;
    let mut logical_processors = 0;
    let mut clock_speed_mhz: f64 = 0.0;

    // Read /proc/cpuinfo
    let file = fs::File::open("/proc/cpuinfo")?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with("model name") {
            cpu_name = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("vendor_id") {
            manufacturer = Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
        } else if line.starts_with("cpu MHz") {
            if let Some(freq_str) = line.split(':').nth(1) {
                clock_speed_mhz = freq_str.trim().parse().unwrap_or(0.0);
            }
        } else if line.starts_with("processor") {
            logical_processors += 1;
        }
    }

    Ok(CpuInfo {
        cpu_name,
        logical_processors,
        processor_family: None, // This information is not readily available in /proc/cpuinfo
        manufacturer,
        clock_speed_mhz: clock_speed_mhz as u64,
        description: None,
    })
}

pub fn get_gpu_info() -> io::Result<Vec<GpuInfo>> {
    let output = Command::new("lspci")
        .args(["-v", "-mm"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "lspci command failed"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();
    let mut current_gpu = None;
    let mut current_driver = None;
    let mut is_vga_controller = false;

    for line in output_str.lines() {
        if line.starts_with("Slot:") {
            // If we have a previous GPU and it's a VGA/3D controller, add it to the list
            if let Some(name) = current_gpu {
                if is_vga_controller {
                    gpus.push(GpuInfo {
                        name,
                        driver_version: current_driver,
                    });
                }
            }
            current_gpu = None;
            current_driver = None;
            is_vga_controller = false;
        } else if line.starts_with("Device:") {
            if let Some(name) = line.split(':').nth(1) {
                current_gpu = Some(name.trim().to_string());
            }
        } else if line.starts_with("Driver:") {
            if let Some(driver) = line.split(':').nth(1) {
                current_driver = Some(driver.trim().to_string());
            }
        } else if line.starts_with("Class:") {
            if let Some(class) = line.split(':').nth(1) {
                let class = class.trim();
                is_vga_controller = class.contains("VGA") || class.contains("3D");
            }
        }
    }

    // Add the last GPU if it's a VGA/3D controller
    if let Some(name) = current_gpu {
        if is_vga_controller {
            gpus.push(GpuInfo {
                name,
                driver_version: current_driver,
            });
        }
    }

    Ok(gpus)
}
