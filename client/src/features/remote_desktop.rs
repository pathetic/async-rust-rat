use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use screenshots::{image, Screen};
use std::io::Cursor;

use common::packets::{RemoteDesktopConfig, RemoteDesktopFrame, MouseClickData, KeyboardInputData, ServerboundPacket};
use crate::handler::send_packet;

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
};
use std::mem::zeroed;

// Shared stop flag for remote desktop streaming
static mut STREAMING_ACTIVE: Option<Arc<AtomicBool>> = None;

pub fn start_remote_desktop(
    config: RemoteDesktopConfig,
) {
    // If already streaming, stop the previous stream
    stop_remote_desktop();

    // Check if the display is valid
    let screens = Screen::all().unwrap_or_default();
    if config.display as usize >= screens.len() {
        return; // Invalid display index
    }

    // Create a new stop flag
    let stop_flag = Arc::new(AtomicBool::new(false));
    
    // Store the stop flag in the static variable
    unsafe {
        STREAMING_ACTIVE = Some(Arc::clone(&stop_flag));
    }
    
    // Get the screen to capture
    let screen = screens[config.display as usize];
    
    // Target frame delay in milliseconds
    let frame_delay = if config.fps > 0 {
        1000 / config.fps as u64
    } else {
        100 // Default to 10 FPS
    };
    
    // Set JPEG quality (1-100)
    let quality = config.quality.clamp(1, 100);

    // Create a config clone for the thread
    let config_clone = config.clone();

    // Start a new thread for streaming
    thread::spawn(move || {
        // Create a new Tokio runtime for this thread
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
            
        // Keep a reference to the stop flag
        let stop_flag = unsafe { STREAMING_ACTIVE.as_ref().unwrap().clone() };
        
        // Stream as long as the stop flag is not set
        while !stop_flag.load(Ordering::Relaxed) {
            let start_time = SystemTime::now();
            
            // Capture the screen
            match screen.capture() {
                Ok(image) => {
                    // Encode the image as JPEG
                    let mut bytes: Vec<u8> = Vec::new();
                    if let Err(_) = image.write_to(
                        &mut Cursor::new(&mut bytes),
                        image::ImageOutputFormat::Jpeg(quality),
                    ) {
                        continue;
                    }
                    
                    // Get the current timestamp
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    
                    // Create the frame
                    let frame = RemoteDesktopFrame {
                        timestamp,
                        display: config_clone.display,
                        data: bytes,
                    };
                    
                    // Create the packet and send it asynchronously using tokio's runtime
                    let packet = ServerboundPacket::RemoteDesktopFrame(frame);
                    
                    // Use our local runtime to send the packet asynchronously
                    if let Err(e) = rt.block_on(send_packet(packet)) {
                        println!("Failed to send remote desktop frame: {}", e);
                    }
                }
                Err(_) => {
                    // If screen capture fails, wait a bit and try again
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
            
            // Calculate how long to sleep to maintain target FPS
            if let Ok(elapsed) = start_time.elapsed() {
                let elapsed_ms = elapsed.as_millis() as u64;
                if elapsed_ms < frame_delay {
                    thread::sleep(Duration::from_millis(frame_delay - elapsed_ms));
                }
            }
        }
    });
}

pub fn stop_remote_desktop() {
    // Set the stop flag to stop the streaming thread
    unsafe {
        if let Some(flag) = STREAMING_ACTIVE.as_ref() {
            flag.store(true, Ordering::Relaxed);
            STREAMING_ACTIVE = None;
        }
    }
} 

pub fn mouse_click(click_data: MouseClickData) {
    // Set cursor position - round the floating point values to integers for the Windows API
    let x = click_data.x;
    let y = click_data.y;

    // First, position the cursor
    unsafe {
        SetCursorPos(x as i32, y as i32);
    }

    // Handle based on click type and action type
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
        
        _ => {} // Ignore other combinations
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
            let caps_state = GetKeyState(VK_CAPITAL as i32) & 1 != 0;
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
        let caps_state = GetKeyState(VK_CAPITAL as i32) & 1 != 0;
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