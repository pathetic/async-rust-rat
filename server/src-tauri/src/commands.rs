/// Commands used internally for communication between connections and channel loop
use common::async_impl::packets::*;
use std::net::SocketAddr;

use tokio::sync::{mpsc::Sender, oneshot::Sender as OSender};
use crate::handlers::FrontClient;
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
    ScreenshotData(SocketAddr, Vec<u8>),

    TakeScreenshot(SocketAddr, String),
    GetClients(OSender<Vec<FrontClient>>),
    GetClient(SocketAddr, OSender<Option<FrontClient>>),
    SetTauriHandle(AppHandle),
    ClientDisconnected(SocketAddr),
}
