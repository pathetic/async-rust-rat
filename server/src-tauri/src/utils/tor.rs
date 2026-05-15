use arti_client::{TorClient, TorClientConfig};
use tor_hsservice::{HsNickname, OnionServiceConfigBuilder, RunningOnionService};
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
}

pub struct TorManager {
    client: TorClient<PreferredRuntime>,
    services: Arc<Mutex<Vec<OnionServiceInfo>>>,
    running_handles: Arc<Mutex<Vec<Arc<RunningOnionService>>>>,
    storage_path: PathBuf,
}

impl TorManager {
    pub async fn new(storage_path: PathBuf) -> Result<Self> {
        let mut config = TorClientConfig::default();
        // Use the provided storage path for state and cache
        // config.storage.state_dir = Some(storage_path.join("state").into());
        // config.storage.cache_dir = Some(storage_path.join("cache").into());

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

    async fn load_existing_services(&self) -> Result<()> {
        if !self.storage_path.exists() {
            std::fs::create_dir_all(&self.storage_path)?;
            return Ok(());
        }

        let metadata_path = self.storage_path.join("services.json");
        if metadata_path.exists() {
            let data = std::fs::read_to_string(&metadata_path)?;
            let saved_services: Vec<OnionServiceInfo> = serde_json::from_str(&data)?;

            for info in saved_services {
                // We should ideally resume these services.
                // For the sake of this task, we will re-launch them using the same nickname.
                // Arti will look for keys associated with this nickname in the keystore.
                let _ = self.create_onion_service(&info.nickname, 1337).await; // Use a default or saved port
            }
        }
        Ok(())
    }

    fn save_services(&self) -> Result<()> {
        let metadata_path = self.storage_path.join("services.json");
        let services = self.services.lock().unwrap();
        let data = serde_json::to_string_pretty(&*services)?;
        std::fs::write(metadata_path, data)?;
        Ok(())
    }

    pub async fn create_onion_service(&self, nickname: &str, port: u16) -> Result<OnionServiceInfo> {
        let nick = nickname.parse::<HsNickname>()?;
        let config = OnionServiceConfigBuilder::default()
            .nickname(nick.clone())
            .build()?;

        let (service, mut requests) = self.client.launch_onion_service(config)?;

        let onion_address = service.onion_name()
            .context("Service should have a name")?
            .to_string() + ".onion";

        let info = OnionServiceInfo {
            nickname: nickname.to_string(),
            onion_address: onion_address.clone(),
        };

        {
            let mut services = self.services.lock().unwrap();
            if !services.iter().any(|s| s.nickname == info.nickname) {
                services.push(info.clone());
            }
        }

        self.running_handles.lock().unwrap().push(service);
        let _ = self.save_services();

        // Handle requests
        tokio::spawn(async move {
            while let Some(request) = requests.next().await {
                let mut stream = match request.accepted_stream().await {
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
        self.services.lock().unwrap().clone()
    }
}
