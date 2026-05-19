use anyhow::{Context, Result};
use arti_client::{TorClient, TorClientConfig};
use futures::StreamExt;
use safelog::DisplayRedacted;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tor_hsservice::config::OnionServiceConfig;
use tor_hsservice::{HsNickname, RunningOnionService};
use tor_rtcompat::PreferredRuntime;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OnionServiceInfo {
    pub nickname: String,
    pub onion_address: String,
    pub port: u16,
    #[serde(default)]
    pub status: Option<String>,
}

pub struct TorManager {
    client: TorClient<PreferredRuntime>,
    services: Arc<Mutex<Vec<OnionServiceInfo>>>,
    running_handles: Arc<Mutex<std::collections::HashMap<String, Arc<RunningOnionService>>>>,
    status_cache: Arc<Mutex<std::collections::HashMap<String, String>>>,
    storage_path: PathBuf,
}

impl TorManager {
    pub async fn new(storage_path: PathBuf, app_handle: &tauri::AppHandle, server_port: Option<u16>) -> Result<Self> {
        let state_dir = storage_path.join("state");
        let cache_dir = storage_path.join("cache");

        let mut builder = TorClientConfig::builder();
        builder
            .storage()
            .state_dir(arti_client::config::CfgPath::new(
                state_dir.to_string_lossy().into_owned(),
            ))
            .cache_dir(arti_client::config::CfgPath::new(
                cache_dir.to_string_lossy().into_owned(),
            ));

        // Increase the primary guard pool so arti has more candidates when
        // some guards are unreachable.  The default of 3 means a single bad
        // guard can stall all onion circuit builds for 30 seconds at a time.
        // Note: guard-n-primary-guards-to-use-if-max-filtered is NOT a valid
        // consensus param in arti 0.42 — only guard-n-primary-guards is.
        builder.override_net_params().insert("guard-n-primary-guards".to_string(), 6);

        let config = builder.build()?;

        let client = TorClient::builder()
            .config(config)
            .create_unbootstrapped()?;

        let mut events = client.bootstrap_events();
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(status) = events.next().await {
                let _ = app_handle_clone.emit("tor_bootstrap", status.as_frac());
            }
        });

        let client = match client.bootstrap().await {
            Ok(()) => client,
            Err(e) => {
                let err_msg = format!("{}", e);
                eprintln!(
                    "[TorManager] Initial bootstrap failed: {}. Clearing cache and guard state...",
                    err_msg
                );

                // Clear cache and guard state to force fresh guard discovery
                let _ = tokio::fs::remove_dir_all(&cache_dir).await;
                if let Ok(mut entries) = tokio::fs::read_dir(&state_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        // Keep hidden service keys (hss/) but remove guard/consensus state
                        if name_str == "hss" {
                            continue;
                        }
                        let path = entry.path();
                        if path.is_dir() {
                            let _ = tokio::fs::remove_dir_all(&path).await;
                        } else {
                            let _ = tokio::fs::remove_file(&path).await;
                        }
                    }
                }

                let mut retry_builder = TorClientConfig::builder();
                retry_builder
                    .storage()
                    .state_dir(arti_client::config::CfgPath::new(
                        state_dir.to_string_lossy().into_owned(),
                    ))
                    .cache_dir(arti_client::config::CfgPath::new(
                        cache_dir.to_string_lossy().into_owned(),
                    ));
                let retry_config = retry_builder.build()?;

                let retry_client = TorClient::builder()
                    .config(retry_config)
                    .create_unbootstrapped()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Failed to create Tor client after state reset: {}. \
                             Try deleting tor_data/ manually.",
                            e
                        )
                    })?;

                let mut retry_events = retry_client.bootstrap_events();
                let app_h = app_handle.clone();
                tokio::spawn(async move {
                    while let Some(status) = retry_events.next().await {
                        let _ = app_h.emit("tor_bootstrap", status.as_frac());
                    }
                });

                retry_client.bootstrap().await.map_err(|e| {
                    anyhow::anyhow!(
                        "Tor bootstrap failed even after clearing state: {}. \
                         This usually means Tor guard relays are unreachable from your network. \
                         Check your firewall, internet connection, and system clock.",
                        e
                    )
                })?;
                retry_client
            }
        };

        let manager = Self {
            client,
            services: Arc::new(Mutex::new(Vec::new())),
            running_handles: Arc::new(Mutex::new(std::collections::HashMap::new())),
            status_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            storage_path: storage_path.clone(),
        };

        manager.load_existing_services(app_handle, server_port).await?;

        // Spawn the circuit-pool probe as a background task so TorManager::new()
        // returns immediately and the frontend is unblocked.  The probe runs
        // independently and emits tor_hspool_warmup events as it goes.
        {
            use arti_client::StreamPrefs;
            use arti_client::config::BoolOrAuto;

            let services_snapshot = manager.get_services();
            let probe_client = manager.get_tor_client();
            let app_handle_bg = app_handle.clone();

            tokio::spawn(async move {
                if let Some(first_svc) = services_snapshot.first() {
                    let onion_addr = first_svc.onion_address.clone();
                    let port = first_svc.port;

                    println!("[TorManager] Background probe started for {}:{}", onion_addr, port);

                    let timeout_secs: u64 = 180;
                    let start = std::time::Instant::now();
                    let mut attempt = 0u32;

                    loop {
                        let elapsed = start.elapsed().as_secs();

                        if elapsed >= timeout_secs {
                            println!("[TorManager] Probe timed out after {}s — service may still become reachable.", timeout_secs);
                            app_handle_bg.emit("tor_hspool_warmup", serde_json::json!({
                                "remaining": 0, "total": timeout_secs, "attempt": attempt,
                            })).ok();
                            break;
                        }

                        let mut prefs = StreamPrefs::new();
                        prefs.connect_to_onion_services(BoolOrAuto::Explicit(true));

                        match probe_client.connect_with_prefs((onion_addr.as_str(), port), &prefs).await {
                            Ok(_) => {
                                println!("[TorManager] Onion service reachable after {}s ({} attempts).", elapsed, attempt + 1);
                                app_handle_bg.emit("tor_hspool_warmup", serde_json::json!({
                                    "remaining": 0, "total": timeout_secs, "attempt": attempt + 1,
                                })).ok();
                                break;
                            }
                            Err(e) => {
                                attempt += 1;
                                let remaining = timeout_secs.saturating_sub(elapsed);
                                println!("[TorManager] Probe attempt {} failed ({}s elapsed): {}", attempt, elapsed, e);
                                app_handle_bg.emit("tor_hspool_warmup", serde_json::json!({
                                    "remaining": remaining, "total": timeout_secs, "attempt": attempt,
                                })).ok();
                                // Wait before retrying — don't hammer the circuit manager
                                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                            }
                        }
                    }
                }
            });
        }

        Ok(manager)
    }

    /// Recursively remove any `.lock` files left behind by a previous unclean shutdown.
    /// Without this, arti refuses to re-launch the same service on restart.
    fn clean_stale_locks(dir: &PathBuf) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::clean_stale_locks(&path);
                } else if path.extension().map_or(false, |e| e == "lock") {
                    let _ = std::fs::remove_file(&path);
                    println!("[TorManager] Removed stale lock: {}", path.display());
                }
            }
        }
    }

    async fn load_existing_services(&self, app_handle: &tauri::AppHandle, server_port: Option<u16>) -> Result<()> {
        if !self.storage_path.exists() {
            tokio::fs::create_dir_all(&self.storage_path).await?;
            return Ok(());
        }

        // Clean up stale lock files from previous unclean shutdown before relaunching services
        Self::clean_stale_locks(&self.storage_path);

        let metadata_path = self.storage_path.join("services.json");
        if metadata_path.exists() {
            let data = tokio::fs::read_to_string(&metadata_path).await?;
            let saved_services: Vec<OnionServiceInfo> = serde_json::from_str(&data)?;

            for info in saved_services {
                // Use the current server port if provided, otherwise use the stored port
                let port_to_use = server_port.unwrap_or(info.port);
                if server_port.is_some() && info.port != port_to_use {
                    println!(
                        "[TorManager] Updating service '{}' port from {} to {} (current server port)",
                        info.nickname, info.port, port_to_use
                    );
                }
                match self
                    .create_onion_service(&info.nickname, port_to_use, app_handle)
                    .await
                {
                    Ok(svc) => println!(
                        "[TorManager] Restored onion service '{}' at {} (forwarding to port {})",
                        svc.nickname, svc.onion_address, port_to_use
                    ),
                    Err(e) => {
                        eprintln!("[TorManager] Failed to restore '{}': {}", info.nickname, e)
                    }
                }
            }
        }
        Ok(())
    }

    async fn save_services(&self) -> Result<()> {
        let metadata_path = self.storage_path.join("services.json");
        let services = {
            let services_guard = self
                .services
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            services_guard.clone()
        };
        let data = serde_json::to_string_pretty(&services)?;
        tokio::fs::write(metadata_path, data).await?;
        Ok(())
    }

    pub async fn create_onion_service(
        &self,
        nickname: &str,
        port: u16,
        app_handle: &tauri::AppHandle,
    ) -> Result<OnionServiceInfo> {
        // Check for duplicates first
        {
            let services = self
                .services
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            if let Some(existing) = services.iter().find(|s| s.nickname == nickname) {
                return Ok(existing.clone());
            }
        }

        let nick = nickname.parse::<HsNickname>()?;
        let config = OnionServiceConfig::builder()
            .nickname(nick.clone())
            .build()?;

        let (service, requests) = self
            .client
            .launch_onion_service(config)?
            .context("Onion services not enabled")?;

        let mut status_events = service.status_events();
        let nickname_clone = nickname.to_string();
        let app_handle_clone = app_handle.clone();
        let status_cache_clone = self.status_cache.clone();
        tokio::spawn(async move {
            while let Some(status) = status_events.next().await {
                // Map arti's internal debug state to a human-readable string.
                // arti's "Running" only means descriptors are published locally —
                // it does NOT mean the service is reachable by clients yet.
                // We add a "CircuitsBuilding" intermediate state to make this clear.
                let raw = format!("{:?}", status.state());
                let display_status = match raw.as_str() {
                    s if s.contains("Bootstrapping") => "Bootstrapping".to_string(),
                    s if s.contains("Running") => {
                        // Check if there are any introduction points established.
                        // If the ipts field is present and non-empty, circuits are up.
                        // Otherwise we're still building them.
                        if s.contains("ipts: []") || s.contains("ipts: None") {
                            "CircuitsBuilding".to_string()
                        } else {
                            "Running".to_string()
                        }
                    }
                    s if s.contains("Shutdown") || s.contains("Dead") => "Shutdown".to_string(),
                    other => other.to_string(),
                };

                if let Ok(mut cache) = status_cache_clone.lock() {
                    cache.insert(nickname_clone.clone(), display_status.clone());
                }
                #[derive(Serialize, Clone)]
                struct OnionStatusEvent {
                    nickname: String,
                    status: String,
                    raw_status: String,
                }
                let _ = app_handle_clone.emit(
                    "tor_service_status",
                    OnionStatusEvent {
                        nickname: nickname_clone.clone(),
                        status: display_status,
                        raw_status: raw,
                    },
                );
            }
        });

        let mut onion_address = service
            .onion_address()
            .context("Service should have a name")?
            .display_unredacted()
            .to_string();

        if !onion_address.ends_with(".onion") {
            onion_address.push_str(".onion");
        }

        let info = OnionServiceInfo {
            nickname: nickname.to_string(),
            onion_address: onion_address.clone(),
            port,
            status: None,
        };

        {
            let mut services = self
                .services
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            services.push(info.clone());
        }

        self.running_handles
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .insert(nickname.to_string(), service);
        let _ = self.save_services().await;

        let nickname_owned = nickname.to_string();

        // Handle requests
        tokio::spawn(async move {
            let mut stream_requests = tor_hsservice::handle_rend_requests(requests);
            while let Some(stream_request) = stream_requests.next().await {
                let mut stream = match stream_request
                    .accept(tor_cell::relaycell::msg::Connected::new_empty())
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[OnionService '{}'] Failed to accept rendezvous stream: {}", nickname_owned, e);
                        continue;
                    }
                };

                let svc_port = port;
                let svc_nick = nickname_owned.clone();
                tokio::spawn(async move {
                    let mut target =
                        match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", svc_port)).await {
                            Ok(t) => t,
                            Err(e) => {
                                eprintln!(
                                    "[OnionService '{}'] Failed to connect to local server port {}: {}",
                                    svc_nick, svc_port, e
                                );
                                return;
                            }
                        };
                    match tokio::io::copy_bidirectional(&mut stream, &mut target).await {
                        Ok((to_target, from_target)) => {
                            println!(
                                "[OnionService '{}'] Stream closed: {} bytes sent, {} bytes received",
                                svc_nick, to_target, from_target
                            );
                        }
                        Err(e) => {
                            eprintln!("[OnionService '{}'] Stream error: {}", svc_nick, e);
                        }
                    }
                });
            }
        });

        Ok(info)
    }

    /// Returns a clone of the underlying TorClient so callers can open new
    /// circuits without needing to bootstrap again.
    pub fn get_tor_client(&self) -> TorClient<PreferredRuntime> {
        self.client.clone()
    }

    pub fn get_services(&self) -> Vec<OnionServiceInfo> {
        let mut svcs = match self.services.lock() {
            Ok(s) => s.clone(),
            Err(e) => e.into_inner().clone(),
        };
        if let Ok(cache) = self.status_cache.lock() {
            for svc in &mut svcs {
                if let Some(status) = cache.get(&svc.nickname) {
                    svc.status = Some(status.clone());
                }
            }
        }
        svcs
    }

    pub async fn delete_onion_service(&self, nickname: &str) -> Result<()> {
        // Drop the running handle → arti stops the service
        {
            let mut handles = self
                .running_handles
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            handles.remove(nickname);
        }

        // Remove from status cache
        {
            let mut cache = self
                .status_cache
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            cache.remove(nickname);
        }

        // Remove from services list
        {
            let mut services = self
                .services
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            services.retain(|s| s.nickname != nickname);
        }

        // Persist updated list
        let _ = self.save_services().await;

        // Remove keys/state so the .onion address is truly deleted
        let hss_dir = self.storage_path.join("state").join("hss").join(nickname);
        if hss_dir.exists() {
            tokio::fs::remove_dir_all(&hss_dir)
                .await
                .unwrap_or_else(|e| eprintln!("[TorManager] Failed to remove hss dir: {}", e));
        }

        println!("[TorManager] Deleted onion service '{}'", nickname);
        Ok(())
    }
}
