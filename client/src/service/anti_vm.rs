use wmi::{COMLibrary, Variant, WMIConnection};
use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use sysinfo::Disks;

pub fn anti_vm_detection() -> bool {
    detect_manufacturer() || detect_debugger() || detect_sandboxie() || is_small_disk()
}

pub fn detect_manufacturer() -> bool {
    let com_con = match COMLibrary::new() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let wmi_con = match WMIConnection::new(com_con) {
        Ok(w) => w,
        Err(_) => return false,
    };

    let results: Vec<HashMap<String, Variant>> =
        match wmi_con.raw_query("SELECT Manufacturer, Model FROM Win32_ComputerSystem") {
            Ok(r) => r,
            Err(_) => return false,
        };

    for item in results {
        let manufacturer = match item.get("Manufacturer") {
            Some(Variant::String(s)) => s.to_lowercase(),
            _ => continue,
        };

        let model = match item.get("Model") {
            Some(Variant::String(s)) => s.to_uppercase(),
            _ => continue,
        };

        if (manufacturer == "microsoft corporation" && model.contains("VIRTUAL"))
            || manufacturer.contains("vmware")
            || model == "VIRTUALBOX"
        {
            return true;
        }
    }
    false
}


type Handle = *mut c_void;
type Bool = i32;

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn LoadLibraryA(lpLibFileName: *const c_char) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
}

fn detect_debugger() -> bool {
    unsafe {
        let module_name = b"kernel32.dll\0";
        let func_name = b"CheckRemoteDebuggerPresent\0";

        let h_module = LoadLibraryA(module_name.as_ptr() as *const c_char);
        if h_module.is_null() {
            return false;
        }

        let func_ptr = GetProcAddress(h_module, func_name.as_ptr() as *const c_char);
        if func_ptr.is_null() {
            return false;
        }

        let check_remote_debugger_present: extern "system" fn(Handle, *mut Bool) -> Bool =
            std::mem::transmute(func_ptr);

        let handle = GetCurrentProcess();
        let mut is_debugger_present: Bool = 0;

        let success = check_remote_debugger_present(handle, &mut is_debugger_present);

        success != 0 && is_debugger_present != 0
    }
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleA(lpModuleName: *const i8) -> *mut core::ffi::c_void;
}

fn detect_sandboxie() -> bool {
    unsafe {
        let dll_name = CString::new("SbieDll.dll").unwrap();
        let handle = GetModuleHandleA(dll_name.as_ptr());
        !handle.is_null()
    }
}

fn is_small_disk() -> bool {
    const GB_60: u64 = 61_000_000_000;

    let exe_path = std::env::current_exe().unwrap_or_else(|_| "/".into());
    let disks = Disks::new_with_refreshed_list();

    for disk in disks.list() {
        let mount_point = disk.mount_point();
        if exe_path.starts_with(mount_point) && disk.total_space() <= GB_60 {
            return true;
        }
    }

    false
}