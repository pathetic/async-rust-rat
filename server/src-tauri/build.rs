use std::env;
use std::fs;
use std::path::PathBuf;

fn restore_rcedit() {
    let source_bkp_path = PathBuf::from("rcedit.bkp");

    if !source_bkp_path.exists() {
        panic!(
            "Error: rcedit.bkp not found at {:?}. Please place it alongside build.rs.",
            source_bkp_path.canonicalize().unwrap_or(source_bkp_path.clone())
        );
    }

    let cargo_manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

    let top_level_target_dir = cargo_manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Could not determine project root.")
        .join("target");


    let destination_exe_path = top_level_target_dir.join("rcedit.exe");

    println!(
        "Attempting to copy {:?} to {:?}",
        source_bkp_path, destination_exe_path
    );

    fs::create_dir_all(&top_level_target_dir)
        .expect(&format!("Failed to create target directory: {:?}", top_level_target_dir));

    if destination_exe_path.exists() {
        println!(
            "rcedit.exe already exists at {:?}. Overwriting.",
            destination_exe_path
        );
        fs::remove_file(&destination_exe_path).expect("Failed to remove existing rcedit.exe");
    }

    fs::copy(&source_bkp_path, &destination_exe_path)
        .expect("Failed to copy rcedit.bkp to rcedit.exe");

    println!("Successfully copied rcedit.bkp to rcedit.exe.");
}

fn main() {
    restore_rcedit();
    tauri_build::build();
}