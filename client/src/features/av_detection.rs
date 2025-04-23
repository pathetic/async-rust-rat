use wmi::{COMLibrary, WMIConnection};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct AntivirusProduct {
    #[serde(rename = "displayName")]
    display_name: String,
}

pub fn get_installed_avs() -> Vec<String> {
    // Initialize COM
    let com_con = match COMLibrary::new() {
        Ok(c) => c,
        Err(e) => {
            return Vec::new();
        },
    };

    // Connect to WMI
    let wmi_con = match WMIConnection::with_namespace_path(r"root\SecurityCenter2", com_con) {
        Ok(c) => c,
        Err(_e) => {
            return Vec::new();
        },
    };

    // Query WMI for antivirus products
    let results: Vec<AntivirusProduct> = match wmi_con.query() {
        Ok(r) => r,
        Err(_e) => {
            return Vec::new();
        },
    };

    // Extract antivirus names
    let av_names: Vec<String> = results
        .into_iter()
        .map(|av| av.display_name)
        .collect();

    av_names
}
