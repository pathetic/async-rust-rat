#[cfg(windows)]
mod windows {
    use std::ptr::null_mut;
    use winapi::um::winuser::{
        SW_HIDE, SW_SHOW, ShowWindow, FindWindowA, FindWindowExA,
        GetDesktopWindow, SetForegroundWindow, GetShellWindow,
        GetForegroundWindow, keybd_event,
        PostMessageA, SystemParametersInfoA,
        VK_ESCAPE, KEYEVENTF_KEYUP, WM_SYSCOMMAND, SC_MONITORPOWER,
    };
    use winapi::shared::minwindef::{TRUE, FALSE};
    use std::process::Command;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::iter::once;
    use std::thread;
    use winapi::um::shellapi::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};

    pub fn toggle_desktop(show: bool) {
        let visibility = if show { SW_SHOW } else { SW_HIDE };
        
        if !try_toggle_desktop_icons(visibility) && !try_toggle_desktop_window(visibility) {
            {}
        }
    }

    pub fn try_toggle_desktop_icons(visibility: i32) -> bool {
        unsafe {
            let prog_man = GetShellWindow();
            if prog_man.is_null() {
                return false;
            }
            
            let shell_view = FindWindowExA(
                prog_man,
                null_mut(),
                b"SHELLDLL_DefView\0".as_ptr() as *const i8,
                null_mut()
            );
            
            if !shell_view.is_null() {
                let desktop_icons = FindWindowExA(
                    shell_view,
                    null_mut(),
                    b"SysListView32\0".as_ptr() as *const i8,
                    b"FolderView\0".as_ptr() as *const i8
                );
                
                if !desktop_icons.is_null() {
                    ShowWindow(desktop_icons, visibility);
                    
                    // If showing, also bring to foreground
                    if visibility == SW_SHOW {
                        SetForegroundWindow(desktop_icons);
                    }
                    
                    return true;
                }
            }
            
            let mut worker_w = FindWindowExA(
                null_mut(),
                null_mut(),
                b"WorkerW\0".as_ptr() as *const i8,
                null_mut()
            );
            
            while !worker_w.is_null() {
                let shell_view = FindWindowExA(
                    worker_w,
                    null_mut(),
                    b"SHELLDLL_DefView\0".as_ptr() as *const i8,
                    null_mut()
                );
                
                if !shell_view.is_null() {
                    let desktop_icons = FindWindowExA(
                        shell_view,
                        null_mut(),
                        b"SysListView32\0".as_ptr() as *const i8,
                        b"FolderView\0".as_ptr() as *const i8
                    );
                    
                    if !desktop_icons.is_null() {
                        ShowWindow(desktop_icons, visibility);
                        
                        if visibility == SW_SHOW {
                            SetForegroundWindow(desktop_icons);
                        }
                        return true;
                    }
                }
                
                worker_w = FindWindowExA(
                    null_mut(),
                    worker_w,
                    b"WorkerW\0".as_ptr() as *const i8,
                    null_mut()
                );
            }
        }
        false
    }

    pub fn try_toggle_desktop_window(visibility: i32) -> bool {
        unsafe {
            let mut worker_w = FindWindowExA(
                null_mut(),
                null_mut(),
                b"WorkerW\0".as_ptr() as *const i8,
                null_mut()
            );
            
            while !worker_w.is_null() {
                let shell_view = FindWindowExA(
                    worker_w,
                    null_mut(),
                    b"SHELLDLL_DefView\0".as_ptr() as *const i8,
                    null_mut()
                );
                
                if !shell_view.is_null() {
                    ShowWindow(worker_w, visibility);
                    
                    if visibility == SW_SHOW {
                        SetForegroundWindow(worker_w);
                    }
                    
                    return true;
                }
                
                worker_w = FindWindowExA(
                    null_mut(),
                    worker_w,
                    b"WorkerW\0".as_ptr() as *const i8,
                    null_mut()
                );
            }
            
            let prog_man = GetShellWindow();
            if !prog_man.is_null() {
                ShowWindow(prog_man, visibility);
                
                // If showing, also bring to foreground
                if visibility == SW_SHOW {
                    SetForegroundWindow(prog_man);
                }
                
                return true;
            }
        }
        false
    }

    pub fn toggle_taskbar(show: bool) {
        unsafe {
            let taskbar = FindWindowA(b"Shell_TrayWnd\0".as_ptr() as *const i8, null_mut());
            
            if !taskbar.is_null() {
                let visibility = if show { SW_SHOW } else { SW_HIDE };
                ShowWindow(taskbar, visibility);
            }
        }
    }

    pub fn toggle_notification_area(show: bool) {
        unsafe {
            let taskbar = FindWindowA(b"Shell_TrayWnd\0".as_ptr() as *const i8, null_mut());
            
            if !taskbar.is_null() {
                let tray_notify_wnd = FindWindowExA(
                    taskbar,
                    null_mut(),
                    b"TrayNotifyWnd\0".as_ptr() as *const i8,
                    null_mut()
                );
                
                if !tray_notify_wnd.is_null() {
                    let visibility = if show { SW_SHOW } else { SW_HIDE };
                    let _action_str = if show { "showing" } else { "hiding" };
                    
                    ShowWindow(tray_notify_wnd, visibility);
                    return;
                }
            }
        }
    }

    pub fn focus_desktop() {
        unsafe {
            const VK_LWIN: u8 = 0x5B;
            const VK_D: u8 = 0x44;
            
            keybd_event(VK_LWIN, 0, 0, 0);
            keybd_event(VK_D, 0, 0, 0);
            keybd_event(VK_D, 0, KEYEVENTF_KEYUP, 0);
            keybd_event(VK_LWIN, 0, KEYEVENTF_KEYUP, 0);
            
            thread::sleep(std::time::Duration::from_millis(100));
            
            let desktop_window = GetDesktopWindow();
            if !desktop_window.is_null() {
                SetForegroundWindow(desktop_window);
                keybd_event(VK_ESCAPE as u8, 0, 0, 0);
                keybd_event(VK_ESCAPE as u8, 0, KEYEVENTF_KEYUP, 0);
            }
        }
    }

    pub fn empty_recycle_bin() {
        unsafe {
            let null_string: Vec<u16> = OsStr::new("")
                .encode_wide()
                .chain(once(0))
                .collect();
            
            let _result = SHEmptyRecycleBinW(
                null_mut(),
                null_string.as_ptr(),
                SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND
            );
        }
    }

    pub fn toggle_invert_mouse(invert: bool) {
        unsafe {
            const SPI_SETMOUSEBUTTONSWAP: u32 = 0x0021;
            
            let swap_value = if invert { TRUE } else { FALSE };
            
            let result = SystemParametersInfoA(
                SPI_SETMOUSEBUTTONSWAP,
                swap_value as u32,
                null_mut(),
                0
            );
            
            if result != TRUE {
                let value = if invert { "1" } else { "0" };
                let _output = Command::new("cmd")
                    .args(["/C", &format!("reg add \"HKCU\\Control Panel\\Mouse\" /v SwapMouseButtons /t REG_SZ /d {} /f", value)])
                    .output();
            }
        }
    }

    pub fn toggle_monitor(on: bool) {
        unsafe {
            let power_state = if on { -1 } else { 2 };
            
            // 1. Try using the foreground window
            let hwnd = GetForegroundWindow();
            if !hwnd.is_null() {
                PostMessageA(hwnd, WM_SYSCOMMAND, SC_MONITORPOWER as usize, power_state as isize);
                return;
            }
            
            // 2. Try using the desktop window
            let hwnd = GetDesktopWindow();
            if !hwnd.is_null() {
                PostMessageA(hwnd, WM_SYSCOMMAND, SC_MONITORPOWER as usize, power_state as isize);
            }
        }
    }
}

#[cfg(unix)]
mod unix {
    pub fn toggle_desktop(_show: bool) {}
    pub fn try_toggle_desktop_icons(_visibility: i32) -> bool { false }
    pub fn try_toggle_desktop_window(_visibility: i32) -> bool { false }
    pub fn toggle_taskbar(_show: bool) {}
    pub fn toggle_notification_area(_show: bool) {}
    pub fn focus_desktop() {}
    pub fn empty_recycle_bin() {}
    pub fn toggle_invert_mouse(_invert: bool) {}
    pub fn toggle_monitor(_on: bool) {}
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;