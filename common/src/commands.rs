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

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptionRequestData {
    pub public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptionResponseData {
    pub secret: Vec<u8>,
}