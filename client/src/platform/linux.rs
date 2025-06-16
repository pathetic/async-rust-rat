use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use std::io::{Cursor, Write};
use std::process::Command;
use std::fs::{OpenOptions, File};
use std::os::unix::fs::OpenOptionsExt;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, GetImageRequest, ImageFormat, Screen, Window};
use x11rb::rust_connection::RustConnection;
use image::{ImageOutputFormat, RgbImage};
use x11rb::protocol::randr::{self, ConnectionExt as RandrExt};

// Mock types to match Windows API
pub type HWND = *mut std::ffi::c_void;
pub type HDC = *mut std::ffi::c_void;
pub type HBITMAP = *mut std::ffi::c_void;
pub type HANDLE = *mut std::ffi::c_void;

// Constants
pub const SW_SHOW: i32 = 5;
pub const SW_HIDE: i32 = 0;
pub const HIDE: u32 = 0;
pub const DETACH: u32 = 0;

// --- Display Handling ---

pub fn get_displays() -> Vec<(i32, i32, i32, i32)> {
    let (conn, screen_num) = match RustConnection::connect(None) {
        Ok((c, sn)) => (c, sn),
        Err(_) => return vec![],
    };
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Query RandR for monitor info
    let res = match conn.randr_get_monitors(root, true) {
        Ok(cookie) => cookie.reply().ok(),
        Err(_) => None,
    };
    if let Some(monitors) = res {
        if !monitors.monitors.is_empty() {
            return monitors.monitors.iter().map(|m| {
                (m.x as i32, m.y as i32, m.width as i32, m.height as i32)
            }).collect();
        }
    }
    // Fallback: single virtual screen
    vec![(0, 0, screen.width_in_pixels as i32, screen.height_in_pixels as i32)]
}

pub fn capture_screen(display_index: usize) -> Option<(Vec<u8>, usize, usize)> {
    capture_display(display_index)
}

pub fn capture_display(display_index: usize) -> Option<(Vec<u8>, usize, usize)> {
    let (conn, screen_num) = RustConnection::connect(None).ok()?;
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Get monitor geometry
    let monitors = conn.randr_get_monitors(root, true).ok()?.reply().ok()?.monitors;
    let (x, y, width, height) = if display_index < monitors.len() {
        let m = &monitors[display_index];
        (m.x as i16, m.y as i16, m.width as u16, m.height as u16)
    } else {
        (0, 0, screen.width_in_pixels as u16, screen.height_in_pixels as u16)
    };

    // Get image in Z_PIXMAP format
    let image_reply = match conn.get_image(
        ImageFormat::Z_PIXMAP,
        root,
        x,
        y,
        width,
        height,
        u32::MAX,
    ) {
        Ok(reply) => match reply.reply() {
            Ok(reply) => reply,
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    let data = image_reply.data;
    let pixel_count = (width as usize * height as usize);
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    
    // Convert BGRA to RGB - we know the data is valid since it comes from X11
    rgb_data.extend(
        data.chunks_exact(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .flatten()
    );

    Some((rgb_data, width as usize, height as usize))
}

pub fn get_primary_display() -> usize {
    0
}

// Desktop manipulation mock implementations
pub fn toggle_desktop(show: bool) {
    // No-op for Linux
}

pub fn toggle_taskbar(show: bool) {
    // No-op for Linux
}

pub fn toggle_notification_area(show: bool) {
    // No-op for Linux
}

pub fn focus_desktop() {
    // No-op on Linux
}

pub fn empty_recycle_bin() {
    // No-op on Linux
}

pub fn toggle_invert_mouse(invert: bool) {
    // No-op for Linux
}

pub fn toggle_monitor(on: bool) {
    // No-op for Linux
}

// Process manipulation mock implementations
pub fn suspend_process(pid: usize) {
    // No-op for Linux
}

pub fn resume_process(pid: usize) {
    // No-op for Linux
}

// Mouse and keyboard input mock implementations
pub fn set_cursor_pos(x: i32, y: i32) {
    // No-op for Linux
}

pub fn send_mouse_event(event_type: u32, x: i32, y: i32) {
    // No-op for Linux
}

pub fn send_keyboard_event(key_code: u16, flags: u32) {
    // No-op for Linux
}

// System info collection mock implementations
pub async fn collect_system_info() -> common::client_info::SystemInfo {
    get_system_info()
}

// Tray icon mock implementation
pub struct TrayIcon {
    unattended: bool,
}

impl Default for TrayIcon {
    fn default() -> Self {
        TrayIcon { unattended: true }
    }
}

impl TrayIcon {
    pub fn new() -> Self {
        TrayIcon { unattended: true }
    }

    pub fn set_unattended(&mut self, unattended: bool) {
        self.unattended = unattended;
    }

    pub fn show(&mut self) {
        // No-op on Linux
    }

    pub fn hide(&mut self) {
        // No-op on Linux
    }
}

pub fn toggle_screen_saver(enable: bool) {
    // No-op on Linux
    let _ = enable;
}

fn get_linux_os_info() -> (String, String) {
    // Try /etc/os-release first
    if let Ok(os_info) = std::fs::read_to_string("/etc/os-release") {
        let name = os_info
            .lines()
            .find(|line| line.starts_with("PRETTY_NAME="))
            .and_then(|line| line.split_once('='))
            .map(|(_, value)| value.trim_matches('"').to_string())
            .unwrap_or_else(|| "Linux".to_string());

        let version = os_info
            .lines()
            .find(|line| line.starts_with("VERSION_ID="))
            .and_then(|line| line.split_once('='))
            .map(|(_, value)| value.trim_matches('"').to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        return (name, version);
    }

    // Try /etc/lsb-release as fallback
    if let Ok(lsb_info) = std::fs::read_to_string("/etc/lsb-release") {
        let name = lsb_info
            .lines()
            .find(|line| line.starts_with("DISTRIB_DESCRIPTION="))
            .and_then(|line| line.split_once('='))
            .map(|(_, value)| value.trim_matches('"').to_string())
            .unwrap_or_else(|| "Linux".to_string());

        let version = lsb_info
            .lines()
            .find(|line| line.starts_with("DISTRIB_RELEASE="))
            .and_then(|line| line.split_once('='))
            .map(|(_, value)| value.trim_matches('"').to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        return (name, version);
    }

    // If all else fails, try to get basic Linux info
    let uname = Command::new("uname")
        .args(["-a"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "Linux".to_string());

    (uname, "Unknown".to_string())
}

fn get_username() -> String {
    Command::new("whoami")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

fn get_hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

fn get_system_model() -> String {
    std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .unwrap_or_else(|_| "Linux PC".to_string())
        .trim()
        .to_string()
}

fn get_system_manufacturer() -> String {
    std::fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor")
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string()
}

pub fn get_system_info() -> common::client_info::SystemInfo {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    
    let (os_name, os_version) = get_linux_os_info();
    
    common::client_info::SystemInfo {
        machine_name: get_hostname(),
        username: get_username(),
        system_model: Some(get_system_model()),
        system_manufacturer: Some(get_system_manufacturer()),
        os_full_name: Some(os_name),
        os_version: Some(os_version),
        os_serial_number: Some("N/A".to_string()), // Linux doesn't have a standard OS serial number
        is_elevated: is_elevated(),
    }
}

pub fn is_elevated() -> bool {
    // Basic check for root privileges
    unsafe { libc::geteuid() == 0 }
}

pub fn toggle_keyboard(enable: bool) {
    // No-op for Linux
}

pub fn visit_website(url: &str) {
    // No-op for Linux
}

pub fn show_messagebox(message: &str) {
    // No-op for Linux
}

pub fn show_input_dialog(prompt: &str) -> Option<String> {
    // Return None for Linux
    None
}

pub fn handle_input_command(input_box_data: &str) {
    // No-op for Linux
}
