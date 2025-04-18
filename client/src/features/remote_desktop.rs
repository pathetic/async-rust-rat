use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use screenshots::{image, Screen};
use std::io::Cursor;

use common::packets::{RemoteDesktopConfig, RemoteDesktopFrame, MouseClickData, ServerboundPacket};
use crate::handler::send_packet;

use winapi::um::winuser::{
    SetCursorPos, 
    mouse_event, 
    MOUSEEVENTF_LEFTDOWN, 
    MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP,
};

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

    if click_data.click_type == 0 {
        unsafe {
            SetCursorPos(x as i32, y as i32);
            mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
            mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        }
    } else if click_data.click_type == 2 {
        unsafe {
            SetCursorPos(x as i32, y as i32);
            mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
            mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
        }
    }
} 