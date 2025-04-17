use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{ TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY };
use std::ptr;

pub fn is_elevated() -> bool {
    unsafe {
        let mut handle: HANDLE = ptr::null_mut();
        if
            OpenProcessToken(
                winapi::um::processthreadsapi::GetCurrentProcess(),
                TOKEN_QUERY,
                &mut handle
            ) != 0
        {
            let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
            if
                GetTokenInformation(
                    handle,
                    TokenElevation,
                    &mut elevation as *mut _ as *mut _,
                    size,
                    &mut size
                ) != 0
            {
                return elevation.TokenIsElevated != 0;
            }
        }
    }
    false
}