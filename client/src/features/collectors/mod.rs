pub mod bios;
pub mod cpu;
pub mod gpu;
pub mod ram;
pub mod system;
pub mod security;
pub mod drives;
pub mod unique;
pub mod displays;

use common::client_info::{ClientInfo, ClientData};

pub async fn client_info(group: String) -> ClientInfo {
    let client_data = ClientData::init(group);
    let system_info = system::collect_system_info().await;
    let ram_info = ram::collect_ram_info().await;
    let cpu_info = cpu::collect_cpu_info().await;
    let bios_info = bios::collect_bios_info().await;
    let gpus_info = gpu::collect_gpu_info().await;
    let displays_info = displays::get_display_count();
    let drives_info = drives::collect_physical_drives().await;
    let unique_info = unique::collect_unique_info().await;
    let security_info = security::collect_security_info().await;

    ClientInfo {
        data: client_data,
        system: system_info,
        ram: ram_info,
        cpu: cpu_info,
        bios: bios_info,
        gpus: gpus_info,
        displays: displays_info,
        drives: drives_info,
        unique: unique_info,
        security: security_info,
    }
}