pub mod config;
pub mod mutex;

#[cfg(windows)]
pub mod install;
#[cfg(windows)]
pub mod anti_vm;
#[cfg(windows)]
pub mod tray_icon;

#[cfg(unix)]
pub mod install;
#[cfg(unix)]
pub mod anti_vm;