//! Commands used internally for communication between connections and channel loop
use common::async_impl::packets::*;
use std::net::SocketAddr;

use tokio::sync::{mpsc::Sender, oneshot::Sender as OSender};


/// Commands sent to client-server connection handlers.
#[derive(Debug)]
pub enum ClientCommand {
    Write(ClientboundPacket),
    SetSecret(Option<Vec<u8>>),
    Close,
}

/// Commands sent to [`AccordChannel`](`crate::channel::AccordChannel`)
#[derive(Debug)]
pub enum ServerCommand {
    Close,
    Write(ClientboundPacket),
    EncryptionRequest(Sender<ClientCommand>, OSender<Vec<u8>>),
    // Maybe this should be a struct?
    EncryptionConfirm(
        Sender<ClientCommand>,
        OSender<Result<Vec<u8>, ()>>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    ), // encrypted secret, encrypted token and expected token
}
