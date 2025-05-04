use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ClientInfo {
    pub data: ClientData,
    pub system: SystemInfo,
    pub ram: RamInfo,
    pub cpu: CpuInfo,
    pub bios: BiosInfo,
    pub gpus: Vec<GpuInfo>,
    pub displays: usize,
    pub drives: Vec<PhysicalDrive>,
    pub unique: UniqueInfo,
    pub security: SecurityInfo,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ClientData {
    pub uuidv4: Option<String>,
    pub addr: Option<String>,
    pub reverse_proxy_port: String,
    pub disconnected: Option<bool>,
    pub group: String,
    pub country_code: String,
}

impl ClientData {
    pub fn init(group: String) -> Self {
        ClientData {
            uuidv4: None,
            addr: None,
            reverse_proxy_port: "".to_string(),
            disconnected: None,
            group,
            country_code: "N/A".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct BiosInfo {
    pub manufacturer: String,
    pub description: String,
    pub serial_number: String,
    pub version: String,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct CpuInfo {
    pub cpu_name: String,
    pub logical_processors: usize,
    pub processor_family: Option<String>,
    pub manufacturer: Option<String>,
    pub clock_speed_mhz: u64,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct PhysicalDrive {
    pub model: String,
    pub size_gb: f64,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub driver_version: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct RamInfo {
    pub total_gb: f64,
    pub used_gb: f64,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct SecurityInfo {
    pub firewall_enabled: bool,
    pub antivirus_names: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct SystemInfo {
    pub username: String,
    pub machine_name: String,
    pub system_model: Option<String>,
    pub system_manufacturer: Option<String>,
    pub os_full_name: Option<String>,
    pub os_version: Option<String>,
    pub os_serial_number: Option<String>,
    pub is_elevated: bool,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct UniqueInfo {
    pub mac_address: String,
    pub volume_serial: String,
}