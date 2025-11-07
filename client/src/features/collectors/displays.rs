#[cfg(windows)]
mod imp {
    use winapi::um::winuser::EnumDisplayMonitors;
    use winapi::shared::windef::{ HMONITOR, HDC, RECT };
    use winapi::shared::minwindef::{ BOOL, LPARAM };

    use std::ptr;

    unsafe extern "system" fn monitor_enum_proc(
        _h_monitor: HMONITOR,
        _hdc_monitor: HDC,
        _lprc_monitor: *mut RECT,
        lparam: LPARAM
    ) -> BOOL {
        // Cast lparam to *mut usize and increment the counter
        let count_ptr = lparam as *mut usize;
        // Ensure proper alignment before dereferencing
        if count_ptr as usize % std::mem::align_of::<usize>() == 0 {
            *count_ptr += 1;
        }
        1
    }

    pub fn get_display_count() -> usize {
        let mut display_count = 0;

        unsafe {
            EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null_mut(),
            Some(monitor_enum_proc),
                &mut display_count as *mut _ as LPARAM
            );
        }

        display_count
    }
}

#[cfg(unix)]
mod imp {
    use x11rb::connection::Connection;
    use x11rb::rust_connection::RustConnection;
    use x11rb::protocol::randr::ConnectionExt as RandrExt;

    pub fn get_display_count() -> usize {
        let (conn, screen_num) = match RustConnection::connect(None) {
            Ok((c, sn)) => (c, sn),
            Err(_) => return 1, // Fallback to 1 if connection fails
        };
        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let root = screen.root;

        // Query RandR for monitor info
        let res = match conn.randr_get_monitors(root, true) {
            Ok(cookie) => cookie.reply().ok(),
            Err(_) => None,
        };

        match res {
            Some(monitors) if !monitors.monitors.is_empty() => monitors.monitors.len(),
            _ => 1, // Fallback to 1 if no monitors found
        }
    }
}

pub use imp::get_display_count;
