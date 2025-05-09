use std::io::Cursor;
use image::{ImageBuffer, Rgb};

pub fn detect_image_dimensions(frame: &[u8]) -> (u32, u32) {
    let len = frame.len();
    let common_resolutions = [
        (640, 480, 460800),
        (1280, 720, 1382400),
        (1920, 1080, 3110400),
        (2304, 1296, 4478976),
        (2560, 1440, 5529600),
        (3840, 2160, 12441600),
    ];
    
    for &(width, height, expected_size) in &common_resolutions {
        let size_ratio = (len as f64) / (expected_size as f64);
        if len == expected_size || (size_ratio > 0.95 && size_ratio < 1.05) {
            return (width, height);
        }
    }
    
    let estimated_pixels = (len as f64 * 2.0 / 3.0) as u32;
    let width = (estimated_pixels as f64).sqrt() as u32;
    let height = estimated_pixels / width;
    (width, height)
}

pub fn process_webcam_frame(frame: Vec<u8>) -> Result<Vec<u8>, String> {    
    let (width, height) = detect_image_dimensions(&frame);
    
    convert_yuv_to_jpeg(frame, width, height)
}

pub fn convert_yuv_to_jpeg(frame: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut rgb_data = vec![0u8; (width * height * 3) as usize];
    
    let y_plane_size = (width * height) as usize;
    
    if frame.len() < y_plane_size + (y_plane_size / 2) {
        return Err("Frame data too small for NV12 conversion".to_string());
    }
    
    for y in 0..height {
        for x in 0..width {
            let y_index = (y * width + x) as usize;
            let rgb_index = y_index * 3;
            
            let uv_x = (x / 2) as usize;
            let uv_y = (y / 2) as usize;
            let uv_index = y_plane_size + (uv_y * (width as usize / 2) + uv_x) * 2;
            
            let y_value = frame[y_index] as f32;
            
            let mut u_value: f32 = 128.0;
            let mut v_value: f32 = 128.0;
            
            if uv_index + 1 < frame.len() {
                u_value = frame[uv_index] as f32;
                v_value = frame[uv_index + 1] as f32;
            }
            
            let y_normalized = 1.164 * (y_value - 16.0);
            let u_normalized = u_value - 128.0;
            let v_normalized = v_value - 128.0;
            
            let r = y_normalized + 1.596 * v_normalized;
            let g = y_normalized - 0.813 * v_normalized - 0.391 * u_normalized;
            let b = y_normalized + 2.018 * u_normalized;
            
            // Clamp values to valid RGB range
            rgb_data[rgb_index] = r.clamp(0.0, 255.0) as u8;      // R
            rgb_data[rgb_index + 1] = g.clamp(0.0, 255.0) as u8;  // G
            rgb_data[rgb_index + 2] = b.clamp(0.0, 255.0) as u8;  // B
        }
    }
    
    let img_buffer = match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, rgb_data) {
        Some(buffer) => buffer,
        None => {
            return Err("Failed to create image buffer from converted RGB data".to_string());
        }
    };
    
    let mut jpeg_data = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_data);
    if let Err(e) = img_buffer.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(85)) {
        return Err(format!("Failed to encode as JPEG: {}", e));
    }
    
    Ok(jpeg_data)
}