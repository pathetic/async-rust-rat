use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use common::packets::{DesktopRecordingPreviewFrame, FileData, RemoteDesktopConfig, ServerboundPacket};
use crate::features::remote_desktop::capture_screen;
use crate::handler::send_packet;
use image::{imageops::FilterType, DynamicImage, RgbImage};

static DESKTOP_RECORDING_ACTIVE: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));

fn send_desktop_recording_file(name: String, data: Vec<u8>) {
    let _ = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(send_packet(ServerboundPacket::DesktopRecordingFile(FileData {
            name,
            data,
        })));
}

fn send_desktop_preview_frame(display: i32, width: usize, height: usize, data: Vec<u8>) {
    let _ = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(send_packet(ServerboundPacket::DesktopRecordingPreviewFrame(
            DesktopRecordingPreviewFrame {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                display,
                width,
                height,
                data,
            },
        )));
}

pub fn start_desktop_recording(config: RemoteDesktopConfig) {
    stop_desktop_recording();

    let stop_flag = Arc::new(AtomicBool::new(false));
    *DESKTOP_RECORDING_ACTIVE.lock().unwrap() = Some(stop_flag.clone());

    thread::spawn(move || {
        let frame_delay = Duration::from_millis(1000 / config.fps.max(1) as u64);
        let mut index = 0;
        let archive = Cursor::new(Vec::new());
        let mut zipper = ZipWriter::new(archive);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        while !stop_flag.load(Ordering::Relaxed) {
            if let Some((raw_data, width, height)) = capture_screen(config.display) {
                let mut rgb_data = Vec::with_capacity(width * height * 3);
                for chunk in raw_data.chunks(3) {
                    let b = chunk[0];
                    let g = chunk[1];
                    let r = chunk[2];
                    rgb_data.extend_from_slice(&[r, g, b]);
                }

                if let Some(image) = RgbImage::from_raw(width as u32, height as u32, rgb_data) {
                    let dynamic = DynamicImage::ImageRgb8(image.clone());
                    let preview = dynamic.resize(640, 360, FilterType::Lanczos3);
                    let mut preview_bytes = Cursor::new(Vec::new());
                    if image::codecs::jpeg::JpegEncoder::new_with_quality(
                        &mut preview_bytes,
                        config.quality,
                    )
                    .encode_image(&preview)
                    .is_ok()
                    {
                        send_desktop_preview_frame(
                            config.display,
                            preview.width() as usize,
                            preview.height() as usize,
                            preview_bytes.into_inner(),
                        );
                    }

                    let mut jpeg_bytes = Cursor::new(Vec::new());
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                        &mut jpeg_bytes,
                        config.quality,
                    );
                    if encoder.encode_image(&image).is_ok() {
                        let frame_name = format!("frame_{:05}.jpg", index);
                        if zipper.start_file(frame_name, options).is_ok() {
                            let _ = zipper.write_all(&jpeg_bytes.into_inner());
                        }
                        index += 1;
                    }
                }
            }
            thread::sleep(frame_delay);
        }

        if let Ok(archive) = zipper.finish() {
            let data = archive.into_inner();
            let name = format!("desktop_recording_{}.zip", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
            send_desktop_recording_file(name, data);
        }
    });
}

pub fn stop_desktop_recording() {
    if let Some(flag) = DESKTOP_RECORDING_ACTIVE.lock().unwrap().take() {
        flag.store(true, Ordering::Relaxed);
    }
}
