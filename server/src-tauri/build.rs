use std::env;
use std::fs;
use std::path::{PathBuf};
use std::process::{Command, Stdio};

fn restore_assets_for_dev_and_standalone() {
    let cargo_manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

    let project_root = cargo_manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Could not determine project root.");

    #[cfg(target_os = "windows")]
    {
        let batch_path = project_root.join("prepare_prod_build.bat");
        println!("Running batch file: {:?}", batch_path);

        let status = Command::new("cmd")
            .arg("/C")
            .arg(&batch_path)
            .current_dir(&project_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to execute prepare_prod_build.bat");

        if !status.success() {
            panic!(
                "prepare_prod_build.bat failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
        }
    }

    let res_dir = project_root.join("res");
    let target_mode_dir = if cfg!(debug_assertions) {
        project_root.join("target").join("debug")
    } else {
        project_root.join("target").join("release")
    };

    let destination_resources_dir = target_mode_dir.join("resources");

    println!(
        "Creating destination directory for dev/standalone assets: {:?}",
        destination_resources_dir
    );
    fs::create_dir_all(&destination_resources_dir)
        .expect(&format!(
            "Failed to create resources directory: {:?}",
            destination_resources_dir
        ));

    #[cfg(target_os = "windows")]
    {
        let rcedit_source_bkp_path = res_dir.join("rcedit.bkp");
        let rcedit_destination_exe_path = destination_resources_dir.join("rcedit.exe");
        println!(
            "Attempting to copy {:?} to {:?}",
            rcedit_source_bkp_path, rcedit_destination_exe_path
        );
        if !rcedit_source_bkp_path.exists() {
            panic!(
                "Error: rcedit.bkp not found at {:?}. Please place it in your project root's /res/ folder.",
                rcedit_source_bkp_path.canonicalize().unwrap_or(rcedit_source_bkp_path.clone())
            );
        }
        if rcedit_destination_exe_path.exists() {
            println!(
                "rcedit.exe already exists at {:?}. Overwriting.",
                rcedit_destination_exe_path
            );
            fs::remove_file(&rcedit_destination_exe_path)
                .expect("Failed to remove existing rcedit.exe");
        }
        fs::copy(&rcedit_source_bkp_path, &rcedit_destination_exe_path)
            .expect("Failed to copy rcedit.bkp to rcedit.exe");
        println!("Successfully copied rcedit.bkp to rcedit.exe.");
    }

    let countries_mmdb_source_path = res_dir.join("countries.mmdb");
    let countries_mmdb_destination_path = destination_resources_dir.join("countries.mmdb");
    println!(
        "Attempting to copy {:?} to {:?}",
        countries_mmdb_source_path, countries_mmdb_destination_path
    );
    if !countries_mmdb_source_path.exists() {
        panic!(
            "Error: countries.mmdb not found at {:?}. Please place it in your project root's /res/ folder.",
            countries_mmdb_source_path.canonicalize().unwrap_or(countries_mmdb_source_path.clone())
        );
    }
    if countries_mmdb_destination_path.exists() {
        println!(
            "countries.mmdb already exists at {:?}. Overwriting.",
            countries_mmdb_destination_path
        );
        fs::remove_file(&countries_mmdb_destination_path)
            .expect("Failed to remove existing countries.mmdb");
    }
    fs::copy(&countries_mmdb_source_path, &countries_mmdb_destination_path)
        .expect("Failed to copy countries.mmdb to countries.mmdb");
    println!("Successfully copied countries.mmdb to countries.mmdb.");

    let dev_resources_path_str = destination_resources_dir
        .to_str()
        .expect("Failed to convert resources path to string")
        .to_string();
    println!("cargo:rustc-env=DEV_RESOURCES_PATH={}", dev_resources_path_str);

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed={}", rcedit_source_bkp_path.display());
    }
    println!("cargo:rerun-if-changed={}", countries_mmdb_source_path.display());
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed={}", batch_path.display());
    }
}

fn main() {
    restore_assets_for_dev_and_standalone();
    tauri_build::build();
}
