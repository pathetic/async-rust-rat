use common::ClientConfig;

pub fn get_config() -> ClientConfig {
    let mut config: ClientConfig = ClientConfig {
        ip: "127.0.0.1".to_string(),
        port: "1337".to_string(),
        group: "Default".to_string(),

        install: false,
        file_name: "Test.exe".to_string(),
        install_folder: "appdata".to_string(),
        enable_hidden: true,
        anti_vm_detection: false,

        mutex_enabled: false,
        mutex: "TEST123".to_string(),
        unattended_mode: false,
    };

    let config_link_sec: Result<ClientConfig, rmp_serde::decode::Error> = rmp_serde::from_read(
        std::io::Cursor::new(&crate::CONFIG)
    );

    if let Ok(config_link_sec) = config_link_sec.as_ref() {
        config = config_link_sec.clone();
    }

    config
}
