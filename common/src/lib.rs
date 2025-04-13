use serde::{ Serialize, Deserialize };

pub mod buffers;
pub mod commands;

pub const RSA_BITS: usize = 1024;
pub const ENC_TOK_LEN: usize = 32;
pub const SECRET_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientConfig {
    pub ip: String,
    pub port: String,
    pub mutex_enabled: bool,
    pub mutex: String,
    pub unattended_mode: bool,
    pub startup: bool
}