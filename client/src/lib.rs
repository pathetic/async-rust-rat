pub mod features;
pub mod service;
pub mod handler;
pub mod platform;
pub mod globals;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::features::reverse_shell::ReverseShell;

