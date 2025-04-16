extern crate winapi;

use winapi::um::synchapi::CreateMutexW;
use winapi::um::errhandlingapi::GetLastError;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::um::handleapi::CloseHandle;
use std::process::exit;
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;

use winapi::shared::ntdef::HANDLE;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct MutexLock {
    handle: HANDLE,
    mutex_enabled: bool,
    mutex_value: String,
}

impl MutexLock {
    pub fn new() -> Self {
        MutexLock {
            handle: ptr::null_mut(),
            mutex_enabled: false,
            mutex_value: String::new(),
        }
    }

    pub fn init(&mut self, mutex_enabled: bool, mutex_value: String) {
        self.mutex_enabled = mutex_enabled;
        self.mutex_value = mutex_value;

        self.lock();
    }

    pub fn lock(&mut self) {
        if !self.mutex_enabled {
            return;
        }

        let mutex = OsStr::new(&format!("Local\\{}", &self.mutex_value))
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();

        unsafe {
            let mutex_handle = CreateMutexW(ptr::null_mut(), 1, mutex.as_ptr());

            if mutex_handle.is_null() {
                exit(0);
            }

            self.handle = mutex_handle;

            let last_error = GetLastError();
            if last_error == ERROR_ALREADY_EXISTS {
                CloseHandle(mutex_handle);
                exit(0);
            }
        }
    }

    pub fn unlock(&mut self) {
        if !self.mutex_enabled {
            return;
        }

        unsafe {
            CloseHandle(self.handle);
        }
    }
}

unsafe impl Send for MutexLock {}