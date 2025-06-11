use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle};

pub fn get_exe_dir() -> Result<PathBuf, String> {
    let current_exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let exe_dir = current_exe_path.parent()
        .ok_or_else(|| "Could not determine parent directory of current executable".to_string())?;
    Ok(exe_dir.to_path_buf())
}

pub fn get_rcedit_path() -> Result<PathBuf, String> { // _app_handle if not used
    let exe_dir = get_exe_dir()?;
    let resources_subdir = exe_dir.join("resources");
    let rcedit_filename = "rcedit.exe";

    let path_in_resources = resources_subdir.join(rcedit_filename);
    if path_in_resources.exists() {
        return Ok(path_in_resources);
    }

    let path_in_exe_dir = exe_dir.join(rcedit_filename);
    if path_in_exe_dir.exists() {
        return Ok(path_in_exe_dir);
    }

    Err(format!(
        "rcedit.exe not found at {:?} or {:?}. Please ensure it is correctly placed.",
        path_in_resources, path_in_exe_dir
    ))
}

pub fn get_countries_path() -> Result<PathBuf, String> {
    let exe_dir = get_exe_dir()?;
    let resources_subdir = exe_dir.join("resources");
    let countries_filename = "countries.mmdb";

    let path_in_resources = resources_subdir.join(countries_filename);
    if path_in_resources.exists() {
        return Ok(path_in_resources);
    }

    let path_in_exe_dir = exe_dir.join(countries_filename);
    if path_in_exe_dir.exists() {
        return Ok(path_in_exe_dir);
    }

    // If neither is found, return an error
    Err(format!(
        "countries.mmdb not found at {:?} or {:?}. Please ensure it is correctly placed.",
        path_in_resources, path_in_exe_dir
    ))
}

pub fn get_client_exe_path() -> Result<PathBuf, String> {
    let exe_dir = get_exe_dir()?;
    let client_filename = "client.exe";

    let path_in_exe_dir = exe_dir.join(client_filename);
    if path_in_exe_dir.exists() {
        return Ok(path_in_exe_dir);
    }

    let path_in_stub_dir = exe_dir.join("stub").join(client_filename);
    if path_in_stub_dir.exists() {
        return Ok(path_in_stub_dir);
    }

    let error_msg = format!(
        "Original client.exe not found at {:?} or {:?}. This file is needed for 'apply_config'. Please ensure it's built to one of these locations.",
        path_in_exe_dir, path_in_stub_dir
    );
    Err(error_msg)
}

pub fn get_client_built_exe_path() -> Result<PathBuf, String> {
    let current_exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let exe_dir = current_exe_path.parent()
        .ok_or_else(|| "Could not determine parent directory of current executable".to_string())?;

    let path = exe_dir.join("Client_built.exe");

    Ok(path)
}