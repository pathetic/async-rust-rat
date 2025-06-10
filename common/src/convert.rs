pub fn bgra_to_i420(width: usize, height: usize, src: &[u8], dest: &mut Vec<u8>) {
    let stride = src.len() / height;

    dest.clear();

    for y in 0..height {
        for x in 0..width {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let y = (66 * r + 129 * g + 25 * b + 128) / 256 + 16;
            dest.push(clamp(y));
        }
    }

    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let u = (-38 * r - 74 * g + 112 * b + 128) / 256 + 128;
            dest.push(clamp(u));
        }
    }

    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let v = (112 * r - 94 * g - 18 * b + 128) / 256 + 128;
            dest.push(clamp(v));
        }
    }
}

fn clamp(x: i32) -> u8 {
    x.min(255).max(0) as u8
}

pub fn i420_to_rgb(width: usize, height: usize, sy: &[u8], su: &[u8], sv: &[u8], dest: &mut [u8], crop_width: usize, crop_height: usize) {
    let crop_width = crop_width.min(width);
    let crop_height = crop_height.min(height);
    
    let uvw = width >> 1;
    
    if sy.len() < width * height || 
       su.len() < (width >> 1) * (height >> 1) || 
       sv.len() < (width >> 1) * (height >> 1) || 
       dest.len() < crop_width * crop_height * 3 {
        return;
    }
    
    for i in 0..crop_height {
        let sw = i * width;
        let swc = i * crop_width;
        let t = (i >> 1) * uvw;
        
        for j in 0..crop_width {
            let y_index = sw + j;
            let rgbstartc = swc + j;
            let uvi = t + (j >> 1);
            
            let y = sy[y_index] as i32;
            let u = su[uvi] as i32 - 128;
            let v = sv[uvi] as i32 - 128;
            
            let vc = v * 359;
            let uc = u * 88;
            let v2c = v * 182;
            let u2c = u * 453;
            
            let r = y + (vc >> 8);
            let g = y - (uc >> 8) - (v2c >> 8);
            let b = y + (u2c >> 8);
            
            let dst_idx = rgbstartc * 3;
            
            dest[dst_idx] = clamp(r);     // R
            dest[dst_idx + 1] = clamp(g); // G
            dest[dst_idx + 2] = clamp(b); // B
        }
    }
}