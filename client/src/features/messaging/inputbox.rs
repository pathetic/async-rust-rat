use std::ffi::CStr;
use winapi::shared::minwindef::UINT;
use common::packets::{InputBoxData, ServerboundPacket};
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

static mut INPUT_TEXT: [u8; 256] = [0; 256];

// Handle input command — entrypoint
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

// Create and show the input box
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

// Window procedure for input box
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
            let wm_id = loword(wparam as DWORD) as usize;
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

fn loword(l: DWORD) -> WORD {
    (l & 0xffff) as WORD
}