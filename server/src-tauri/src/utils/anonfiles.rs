use reqwest::multipart;
use serde_json::Value;
use anyhow::Result;

pub async fn upload_to_anonfiles(filename: &str, data: Vec<u8>) -> Result<String> {
    let client = reqwest::Client::new();
    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(data).file_name(filename.to_string()));

    let response = client.post("https://anonfilesnew.com/api/upload")
        .multipart(form)
        .send()
        .await?;

    let json: Value = response.json().await?;
    
    if json["status"].as_bool().unwrap_or(false) {
        Ok(json["data"]["file"]["url"]["full"].as_str().unwrap_or("Unknown URL").to_string())
    } else {
        let error_msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
        Err(anyhow::anyhow!("Upload failed: {}", error_msg))
    }
}
