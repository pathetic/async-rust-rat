use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use std::io::Cursor;
use std::ptr::null_mut;

use common::packets::{RemoteDesktopConfig, RemoteDesktopFrame, MouseClickData, KeyboardInputData, ServerboundPacket, ScreenshotData};
use crate::handler::send_packet;

#[cfg(windows)]
use winapi::um::winuser::{
    SetCursorPos, 
    mouse_event, 
    MOUSEEVENTF_LEFTDOWN, 
    MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_WHEEL,
    keybd_event,
    KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE,
    VK_CONTROL,
    VK_MENU,
    VK_SHIFT,
    VK_CAPITAL,
    GetKeyState,
    INPUT,
    INPUT_KEYBOARD,
    KEYBDINPUT,
    SendInput,
    GetSystemMetrics,
    GetDC,
    ReleaseDC,
    CreateCompatibleDC,
    CreateCompatibleBitmap,
    SelectObject,
    BitBlt,
    DeleteObject,
    DeleteDC,
};

#[cfg(windows)]
use winapi::um::wingdi::*;
#[cfg(windows)]
use winapi::um::winuser::*;
#[cfg(windows)]
use winapi::um::winnt::HANDLE;
use image::{ImageOutputFormat, RgbImage};

tokio::task_local! {
    static SEND_RUNTIME: tokio::runtime::Runtime;
}

static STREAMING_ACTIVE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

#[cfg(windows)]
mod windows {
    use super::*;

    pub fn capture_screen() -> Option<(Vec<u8>, usize, usize)> {
        unsafe {
            let hdc_screen = GetDC(null_mut());
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
            let old_obj = SelectObject(hdc_mem, hbitmap as _);

            BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }; 1],
            };

            let mut buffer = vec![0u8; (width * height * 4) as usize];
            GetDIBits(
                hdc_mem,
                hbitmap,
                0,
                height as u32,
                buffer.as_mut_ptr() as *mut _,
                &mut bmi as *mut _ as *mut _,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc_mem, old_obj);
            DeleteObject(hbitmap as HANDLE);
            DeleteDC(hdc_mem);
            ReleaseDC(null_mut(), hdc_screen);

            Some((buffer, width as usize, height as usize))
        }
    }

    pub fn mouse_click(click_data: MouseClickData) {
        // Set cursor position
        let x = click_data.x;
        let y = click_data.y;

        unsafe {
            SetCursorPos(x, y);
        }

        match (click_data.click_type, click_data.action_type) {
            // Left mouse actions
            (0, 0) => unsafe { // Left click (complete)
                mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
            },
            (0, 1) => unsafe { // Left mouse down
                mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
            },
            (0, 2) => unsafe { // Left mouse up
                mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
            },
            
            // Middle mouse actions
            (1, 0) => unsafe { // Middle click (complete)
                mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0);
                mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0);
            },
            (1, 1) => unsafe { // Middle mouse down
                mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0);
            },
            (1, 2) => unsafe { // Middle mouse up
                mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0);
            },
            
            // Right mouse actions
            (2, 0) => unsafe { // Right click (complete)
                mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
            },
            (2, 1) => unsafe { // Right mouse down
                mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
            },
            (2, 2) => unsafe { // Right mouse up
                mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
            },
            
            // Scroll actions
            (3, 4) => unsafe { // Scroll up
                let amount = (click_data.scroll_amount as u32).saturating_mul(120); // WHEEL_DELTA is typically 120
                mouse_event(MOUSEEVENTF_WHEEL, 0, 0, amount, 0);
            },
            (3, 5) => unsafe { // Scroll down
                let amount = (click_data.scroll_amount as u32).saturating_mul(120); // WHEEL_DELTA is typically 120
                // Windows uses two's complement for negative values
                let negative_amount = (0u32).wrapping_sub(amount);
                mouse_event(MOUSEEVENTF_WHEEL, 0, 0, negative_amount, 0);
            },
            
            // For mouse move during drag, we just update the cursor position (already done above)
            (_, 3) => {}, // Just update cursor position without changing button state
            
            _ => {}
        }
    }

    pub fn keyboard_input(input_data: KeyboardInputData) {
        // Handle key presses/releases
        unsafe {
            // Special case for resetting keyboard state (when key_code is 0 and character is empty)
            if input_data.key_code == 0 && input_data.character.is_empty() && !input_data.is_keydown {
                // Force release all modifier keys to ensure clean state
                keybd_event(VK_SHIFT as u8, 0, KEYEVENTF_KEYUP, 0);
                keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
                keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0); // Alt key
                
                // Handle caps lock if needed
                let caps_state = GetKeyState(VK_CAPITAL) & 1 != 0;
                if caps_state != input_data.caps_lock {
                    // Toggle caps lock to match the requested state (off)
                    keybd_event(VK_CAPITAL as u8, 0, 0, 0);
                    keybd_event(VK_CAPITAL as u8, 0, KEYEVENTF_KEYUP, 0);
                }
                
                return; // We've reset the keyboard state, no need to process further
            }
            
            // Handle modifier keys - only press them, don't release them here
            if input_data.shift_pressed {
                keybd_event(VK_SHIFT as u8, 0, 0, 0);
            }
            
            if input_data.ctrl_pressed {
                keybd_event(VK_CONTROL as u8, 0, 0, 0);
            }
            
            // Handle caps lock if needed
            let caps_state = GetKeyState(VK_CAPITAL) & 1 != 0;
            if caps_state != input_data.caps_lock {
                // Toggle caps lock to match the requested state
                keybd_event(VK_CAPITAL as u8, 0, 0, 0);
                keybd_event(VK_CAPITAL as u8, 0, KEYEVENTF_KEYUP, 0);
            }
            
            // Process the main key (character or special key)
            if !input_data.character.is_empty() {
                // For Ctrl+character combinations, use virtual key code approach for common shortcuts
                if input_data.ctrl_pressed && input_data.character.len() == 1 {
                    let c = input_data.character.chars().next().unwrap();
                    // Map common Ctrl+key shortcuts to their virtual key codes
                    let vk = match c.to_ascii_lowercase() {
                        'a' => 0x41, // VK_A
                        'c' => 0x43, // VK_C
                        'v' => 0x56, // VK_V
                        'x' => 0x58, // VK_X
                        'z' => 0x5A, // VK_Z
                        'y' => 0x59, // VK_Y
                        's' => 0x53, // VK_S
                        _ => 0, // Default to 0 for other characters
                    };
                    
                    if vk != 0 {
                        // Use virtual key code approach for this key
                        if input_data.is_keydown {
                            keybd_event(vk as u8, 0, 0, 0);
                        } else {
                            keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
                        }
                    } else {
                        // For other characters, use the Unicode input approach
                        for c in input_data.character.chars() {
                            let mut input: INPUT = zeroed();
                            *input.u.ki_mut() = KEYBDINPUT {
                                wVk: 0,
                                wScan: c as u16,
                                dwFlags: KEYEVENTF_UNICODE | if !input_data.is_keydown { KEYEVENTF_KEYUP } else { 0 },
                                time: 0,
                                dwExtraInfo: 0,
                            };
                            input.type_ = INPUT_KEYBOARD;
                            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
                        }
                    }
                } else {
                    // Regular character input (no Ctrl key)
                    for c in input_data.character.chars() {
                        let mut input: INPUT = zeroed();
                        *input.u.ki_mut() = KEYBDINPUT {
                            wVk: 0,
                            wScan: c as u16,
                            dwFlags: KEYEVENTF_UNICODE | if !input_data.is_keydown { KEYEVENTF_KEYUP } else { 0 },
                            time: 0,
                            dwExtraInfo: 0,
                        };
                        input.type_ = INPUT_KEYBOARD;
                        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
                    }
                }
            } else {
                // For non-printable characters (like arrows, function keys, etc.)
                let key_code = input_data.key_code as u8;
                if input_data.is_keydown {
                    keybd_event(key_code, 0, 0, 0);
                } else {
                    keybd_event(key_code, 0, KEYEVENTF_KEYUP, 0);
                }
            }
            
            // Only release modifier keys on key up events
            if !input_data.is_keydown {
                // Release modifier keys
                if input_data.ctrl_pressed {
                    keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
                }
                
                if input_data.shift_pressed {
                    keybd_event(VK_SHIFT as u8, 0, KEYEVENTF_KEYUP, 0);
                }
            }
        }
    }
}

#[cfg(unix)]
mod unix {
    use common::packets::{MouseClickData, KeyboardInputData};

    pub fn capture_screen() -> Option<(Vec<u8>, usize, usize)> {
        // Return a simple black image
        let width = 800;
        let height = 600;
        let buffer = vec![0u8; width * height * 4];
        Some((buffer, width, height))
    }

    pub fn mouse_click(_click_data: MouseClickData) {
        // No-op implementation
    }

    pub fn keyboard_input(_input_data: KeyboardInputData) {
        // No-op implementation
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;

pub async fn take_screenshot(display: String) {
    let _display_number = display.parse::<i32>().unwrap();

    if let Some((raw_data, w, h)) = capture_screen() {
        let mut rgb_data = Vec::with_capacity(w * h * 3);
        for chunk in raw_data.chunks(3) {
            let b = chunk[0];
            let g = chunk[1];
            let r = chunk[2];
            rgb_data.extend_from_slice(&[r, g, b]);
        }

        let img = RgbImage::from_raw(w as u32, h as u32, rgb_data)
            .expect("Failed to create RGB image");

        let mut jpeg_bytes = Cursor::new(Vec::with_capacity(w * h / 3));
        if img
            .write_to(&mut jpeg_bytes, ImageOutputFormat::Jpeg(75))
            .is_err()
        {
            eprintln!("❌ JPEG compression failed (screenshot)");
            return;
        }

        let jpeg_data = jpeg_bytes.into_inner();

        let screenshot_data = ScreenshotData {
            width: w as u32,
            height: h as u32,
            data: jpeg_data,
        };

        if let Err(e) = send_packet(ServerboundPacket::ScreenshotResult(screenshot_data)).await {
            eprintln!("❌ Failed to send screenshot: {}", e);
        }
    }
}

pub fn start_remote_desktop(config: RemoteDesktopConfig) {
    stop_remote_desktop();

    let stop_flag = Arc::new(AtomicBool::new(false));
    *STREAMING_ACTIVE.lock().unwrap() = Some(Arc::clone(&stop_flag));

    let fps = config.fps.max(1);
    let frame_delay = Duration::from_millis(1000 / fps as u64);
    let config_clone = config;

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let stop_flag = STREAMING_ACTIVE.lock().unwrap().as_ref().unwrap().clone();

        while !stop_flag.load(Ordering::Relaxed) {
            let start_time = SystemTime::now();

            if let Some((raw_data, w, h)) = capture_screen() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                // Convert BGR to RGB
                let mut rgb_data = Vec::with_capacity(w * h * 3);
                for chunk in raw_data.chunks(3) {
                    let b = chunk[0];
                    let g = chunk[1];
                    let r = chunk[2];
                    rgb_data.extend_from_slice(&[r, g, b]);
                }

                // Compress to JPEG
                let img = RgbImage::from_raw(w as u32, h as u32, rgb_data)
                    .expect("Failed to create RGB image");
                let mut jpeg_bytes = Cursor::new(Vec::with_capacity(w * h / 3));
                if img
                    .write_to(&mut jpeg_bytes, ImageOutputFormat::Jpeg(config_clone.quality))
                    .is_err()
                {
                    eprintln!("❌ JPEG compression failed");
                    continue;
                }

                let frame = RemoteDesktopFrame {
                    timestamp,
                    display: config_clone.display,
                    width: w,
                    height: h,
                    data: jpeg_bytes.into_inner(),
                };

                let packet = ServerboundPacket::RemoteDesktopFrame(frame);

                if let Err(e) = rt.block_on(send_packet(packet)) {
                    eprintln!("❌ Failed to send remote desktop frame: {}", e);
                }
            }

            let elapsed = start_time.elapsed().unwrap_or_default();
            if elapsed < frame_delay {
                thread::sleep(frame_delay - elapsed);
            }
        }
    });
}

pub fn stop_remote_desktop() {
    if let Some(flag) = STREAMING_ACTIVE.lock().unwrap().take() {
        flag.store(true, Ordering::Relaxed);
    }
}

pub fn mouse_click(_click_data: MouseClickData) {
    // No-op implementation
}

pub fn keyboard_input(_input_data: KeyboardInputData) {
    // No-op implementation
} 