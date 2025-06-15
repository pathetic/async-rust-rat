#[cfg(windows)]
mod windows {
    extern crate winapi;

    use winapi::um::{synchapi::CreateMutexW, handleapi::CloseHandle, errhandlingapi::GetLastError};
    use std::{ptr, ffi::OsStr, os::windows::ffi::OsStrExt, process::exit};
    use winapi::shared::{winerror::ERROR_ALREADY_EXISTS, ntdef::HANDLE};

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
}

#[cfg(unix)]
mod unix {
    use std::sync::Mutex;
    use std::process::exit;

    pub struct MutexLock {
        mutex: Option<Mutex<()>>,
        mutex_enabled: bool,
        mutex_value: String,
    }

    impl MutexLock {
        pub fn new() -> Self {
            MutexLock {
                mutex: None,
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

            // On Unix, we'll use a simple file-based mutex
            let path = format!("/tmp/{}.lock", self.mutex_value);
            if std::path::Path::new(&path).exists() {
                exit(0);
            }

            // Create the lock file
            if let Err(_) = std::fs::File::create(&path) {
                exit(0);
            }

            self.mutex = Some(Mutex::new(()));
        }

        pub fn unlock(&mut self) {
            if !self.mutex_enabled {
                return;
            }

            // Remove the lock file
            let path = format!("/tmp/{}.lock", self.mutex_value);
            let _ = std::fs::remove_file(path);
            self.mutex = None;
        }
    }

    unsafe impl Send for MutexLock {}
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;