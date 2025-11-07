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
    use std::fs::{File, OpenOptions};
    use std::os::unix::fs::OpenOptionsExt;
    use std::io::Write;
    use std::path::Path;

    pub struct MutexLock {
        lock_file: Option<File>,
        mutex_enabled: bool,
        mutex_value: String,
        lock_path: String,
    }

    impl Default for MutexLock {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MutexLock {
        pub fn new() -> Self {
            MutexLock {
                lock_file: None,
                mutex_enabled: false,
                mutex_value: String::new(),
                lock_path: String::new(),
            }
        }

        pub fn init(&mut self, mutex_enabled: bool, mutex_value: String) {
            self.mutex_enabled = mutex_enabled;
            self.mutex_value = mutex_value;
            self.lock_path = format!("/tmp/{}.lock", self.mutex_value);

            self.lock();
        }

        pub fn lock(&mut self) {
            if !self.mutex_enabled {
                return;
            }

            // Try to create the lock file with exclusive access
            match OpenOptions::new()
                .write(true)
                .create_new(true) // This fails if file already exists
                .open(&self.lock_path) 
            {
                Ok(mut file) => {
                    // Write our PID to the lock file
                    let pid = std::process::id();
                    let _ = writeln!(file, "{}", pid);
                    self.lock_file = Some(file);
                }
                Err(_) => {
                    // Lock file already exists, check if it's stale
                    if let Ok(pid_str) = std::fs::read_to_string(&self.lock_path) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            // Check if the process is still running
                            if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                                // Process is still running, exit
                                exit(0);
                            }
                        }
                    }
                    // Stale lock file, try to remove it and create our own
                    let _ = std::fs::remove_file(&self.lock_path);
                    match OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&self.lock_path) 
                    {
                        Ok(mut file) => {
                            let pid = std::process::id();
                            let _ = writeln!(file, "{}", pid);
                            self.lock_file = Some(file);
                        }
                        Err(_) => {
                            // Still can't create lock file, exit
                            exit(0);
                        }
                    }
                }
            }
        }

        pub fn unlock(&mut self) {
            if !self.mutex_enabled {
                return;
            }

            // Close the lock file
            self.lock_file = None;
            
            // Remove the lock file
            let _ = std::fs::remove_file(&self.lock_path);
        }
    }

    unsafe impl Send for MutexLock {}
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;