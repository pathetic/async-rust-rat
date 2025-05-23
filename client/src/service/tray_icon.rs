use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::mem::{ size_of, zeroed };
use winapi::um::shellapi::{
    NOTIFYICONDATAW,
    NIM_ADD,
    NIM_MODIFY,
    NIM_DELETE,
    NIF_MESSAGE,
    NIF_ICON,
    NIF_TIP,
    NIS_HIDDEN,
};
use winapi::um::winuser::{ WM_APP, LoadIconW };
use std::ptr::null_mut;

const WM_TRAYICON: u32 = WM_APP + 100;

pub struct TrayIcon {
    unattended: bool,
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn new() -> Self {
        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        let h_wnd = winapi::um::wincon::GetConsoleWindow;

        unsafe {
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = h_wnd();
            nid.uID = 1001;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = LoadIconW(null_mut(), winapi::um::winuser::IDI_APPLICATION);
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.dwState = 0;
            nid.dwStateMask = NIS_HIDDEN;
            nid.szTip = [0; 128];
        }

        TrayIcon { unattended: true, nid }
    }

    pub fn set_unattended(&mut self, unattended: bool) {
        self.unattended = unattended;
    }

    pub fn show(&mut self) {
        if self.unattended {
            return;
        }

        unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(NIM_ADD, &mut self.nid);
        }
    }

    pub fn hide(&mut self) {
        if self.unattended {
            return;
        }

        unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(NIM_DELETE, &mut self.nid);
        }
    }

    fn text_to_tooltip(&mut self, text: &str) -> [u16; 128] {
        let mut tray_tool_tip_int: [u16; 128] = [0; 128];
        let tray_tool_tip_str_step: &str = &text.to_string();
        let tray_tool_tip_step_os = OsStr::new(tray_tool_tip_str_step);
        let tray_tool_tip_step_utf16 = tray_tool_tip_step_os.encode_wide().collect::<Vec<u16>>();
        tray_tool_tip_int[..tray_tool_tip_step_utf16.len()].copy_from_slice(
            &tray_tool_tip_step_utf16
        );

        tray_tool_tip_int
    }

    pub fn set_tooltip(&mut self, tooltip: &str) {
        if self.unattended {
            return;
        }

        self.nid.szTip = self.text_to_tooltip(tooltip);
        self.update();
    }

    fn update(&mut self) {
        unsafe {
            winapi::um::shellapi::Shell_NotifyIconW(NIM_MODIFY, &mut self.nid);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        println!(
            "TrayIcon::drop() called.",
        );
        self.hide();
    }
}