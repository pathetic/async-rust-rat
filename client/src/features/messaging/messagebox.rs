use std::ptr::null_mut as NULL;
use winapi::um::winuser;
use common::packets::MessageBoxData;

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