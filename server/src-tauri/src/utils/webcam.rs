use std::io::Cursor;
use image::{ImageBuffer, Rgb};

/// Detect image dimensions based on buffer size
pub fn detect_image_dimensions(frame: &[u8]) -> (u32, u32) {
    let len = frame.len();
    
    // Common webcam resolutions and their expected buffer sizes
    // This assumes YUV/NV12 format (12 bits per pixel = 1.5 bytes)
    let common_resolutions = [
        (640, 480, 460800),     // VGA (640*480*1.5 = 460800)
        (1280, 720, 1382400),   // HD (1280*720*1.5 = 1382400)
        (1920, 1080, 3110400),  // Full HD (1920*1080*1.5 = 3110400)
        (2304, 1296, 4478976),  // Your camera's resolution (2304*1296*1.5 = 4478976)
        (2560, 1440, 5529600),  // QHD (2560*1440*1.5 = 5529600)
        (3840, 2160, 12441600), // 4K (3840*2160*1.5 = 12441600)
    ];
    
    // Try to match with common resolutions
    for &(width, height, expected_size) in &common_resolutions {
        let size_ratio = (len as f64) / (expected_size as f64);
        if len == expected_size || (size_ratio > 0.95 && size_ratio < 1.05) {
            return (width, height);
        }
    }
    
    // If no match, try to calculate based on YUV/NV12 format
    let estimated_pixels = (len as f64 * 2.0 / 3.0) as u32; // Convert from 12-bit to pixel count
    let width = (estimated_pixels as f64).sqrt() as u32;
    let height = estimated_pixels / width;
    (width, height)
}

/// Process webcam frame to JPEG image
pub fn process_webcam_frame(frame: Vec<u8>) -> Result<Vec<u8>, String> {    
    // Detect dimensions of the raw frame
    let (width, height) = detect_image_dimensions(&frame);
    
    // Convert YUV/NV12 to RGB and then to JPEG
    convert_yuv_to_jpeg(frame, width, height)
}

/// Convert NV12/YUV420 format to JPEG with improved color handling
pub fn convert_yuv_to_jpeg(frame: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, String> {
    // Create RGB buffer for the converted image
    let mut rgb_data = vec![0u8; (width * height * 3) as usize];
    
    // NV12 format - Y plane followed by interleaved UV plane
    let y_plane_size = (width * height) as usize;
    
    // Make sure we have enough data for NV12 format
    if frame.len() < y_plane_size + (y_plane_size / 2) {
        return Err("Frame data too small for NV12 conversion".to_string());
    }
    
    // More accurate NV12 to RGB conversion
    for y in 0..height {
        for x in 0..width {
            let y_index = (y * width + x) as usize;
            let rgb_index = y_index * 3;
            
            // Calculate UV indices - UV is at half resolution in both dimensions
            let uv_x = (x / 2) as usize;
            let uv_y = (y / 2) as usize;
            let uv_index = y_plane_size + (uv_y * (width as usize / 2) + uv_x) * 2;
            
            // Get Y, U, V values
            let y_value = frame[y_index] as f32;
            
            // Use grayscale as fallback if we don't have valid UV data
            let mut u_value: f32 = 128.0;
            let mut v_value: f32 = 128.0;
            
            // Only try to read U/V if they're within bounds
            if uv_index + 1 < frame.len() {
                // In NV12, U and V are interleaved in the UV plane
                u_value = frame[uv_index] as f32;
                v_value = frame[uv_index + 1] as f32;
            }
            
            // Standard YUV to RGB conversion formula (BT.601)
            // R = Y + 1.402 * (V - 128)
            // G = Y - 0.344136 * (U - 128) - 0.714136 * (V - 128)
            // B = Y + 1.772 * (U - 128)
            
            // Adjusted YUV to RGB conversion with better constants
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
    
    // Create image buffer from RGB data
    let img_buffer = match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, rgb_data) {
        Some(buffer) => buffer,
        None => {
            return Err("Failed to create image buffer from converted RGB data".to_string());
        }
    };
    
    // Encode as JPEG
    let mut jpeg_data = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_data);
    if let Err(e) = img_buffer.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(85)) {
        return Err(format!("Failed to encode as JPEG: {}", e));
    }
    
    Ok(jpeg_data)
}