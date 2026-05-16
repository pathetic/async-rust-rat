use arti_client::{TorClient, TorClientConfig};
use tor_hsservice::{HsNickname, RunningOnionService};
use tor_hsservice::config::OnionServiceConfig;
use safelog::DisplayRedacted;
use tor_rtcompat::PreferredRuntime;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use anyhow::{Context, Result};
use futures::StreamExt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OnionServiceInfo {
    pub nickname: String,
    pub onion_address: String,
    pub port: u16,
}

pub struct TorManager {
    client: TorClient<PreferredRuntime>,
    services: Arc<Mutex<Vec<OnionServiceInfo>>>,
    running_handles: Arc<Mutex<Vec<Arc<RunningOnionService>>>>,
    storage_path: PathBuf,
}

impl TorManager {
    pub async fn new(storage_path: PathBuf) -> Result<Self> {
        let state_dir = storage_path.join("state");
        let cache_dir = storage_path.join("cache");

        let mut builder = TorClientConfig::builder();
        builder.storage()
            .state_dir(arti_client::config::CfgPath::new(state_dir.to_string_lossy().into_owned()))
            .cache_dir(arti_client::config::CfgPath::new(cache_dir.to_string_lossy().into_owned()));
        let config = builder.build()?;

        let client = TorClient::create_bootstrapped(config).await?;

        let manager = Self {
            client,
            services: Arc::new(Mutex::new(Vec::new())),
            running_handles: Arc::new(Mutex::new(Vec::new())),
            storage_path: storage_path.clone(),
        };

        manager.load_existing_services().await?;

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

    async fn load_existing_services(&self) -> Result<()> {
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
                match self.create_onion_service(&info.nickname, info.port).await {
                    Ok(svc) => println!("[TorManager] Restored onion service '{}' at {}.onion", svc.nickname, svc.onion_address),
                    Err(e) => eprintln!("[TorManager] Failed to restore '{}': {}", info.nickname, e),
                }
            }
        }
        Ok(())
    }

    async fn save_services(&self) -> Result<()> {
        let metadata_path = self.storage_path.join("services.json");
        let services = {
            let services_guard = self.services.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            services_guard.clone()
        };
        let data = serde_json::to_string_pretty(&services)?;
        tokio::fs::write(metadata_path, data).await?;
        Ok(())
    }

    pub async fn create_onion_service(&self, nickname: &str, port: u16) -> Result<OnionServiceInfo> {
        // Check for duplicates first
        {
            let services = self.services.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            if let Some(existing) = services.iter().find(|s| s.nickname == nickname) {
                return Ok(existing.clone());
            }
        }

        let nick = nickname.parse::<HsNickname>()?;
        let config = OnionServiceConfig::builder()
            .nickname(nick.clone())
            .build()?;

        let (service, requests) = self.client.launch_onion_service(config)?
            .context("Onion services not enabled")?;

        let onion_address = service.onion_address()
            .context("Service should have a name")?
            .display_unredacted()
            .to_string(); // No ".onion" suffix, it's included in Display

        let info = OnionServiceInfo {
            nickname: nickname.to_string(),
            onion_address: onion_address.clone(),
            port,
        };

        {
            let mut services = self.services.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            services.push(info.clone());
        }

        self.running_handles.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?.push(service);
        let _ = self.save_services().await;

        // Handle requests
        tokio::spawn(async move {
            let mut stream_requests = tor_hsservice::handle_rend_requests(requests);
            while let Some(stream_request) = stream_requests.next().await {
                let mut stream = match stream_request.accept(tor_cell::relaycell::msg::Connected::new_empty()).await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                tokio::spawn(async move {
                    let mut target = match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    let _ = tokio::io::copy_bidirectional(&mut stream, &mut target).await;
                });
            }
        });

        Ok(info)
    }

    pub fn get_services(&self) -> Vec<OnionServiceInfo> {
        match self.services.lock() {
            Ok(s) => s.clone(),
            Err(e) => e.into_inner().clone()
        }
    }
}
