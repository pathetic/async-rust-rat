use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn restore_assets_for_dev_and_standalone() {
    let cargo_manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

    // Determine the project root (what we've been calling 'workspace_root')
    // CARGO_MANIFEST_DIR is `server/src-tauri/`.
    // Go up two levels to `your-main-project-root/`.
    let project_root = cargo_manifest_dir
        .parent() // Go up from `src-tauri` to `server`
        .and_then(|p| p.parent()) // Go up from `server` to `your-main-project-root` (project root)
        .expect("Could not determine project root.");

    // --- Define Source Paths ---
    // Both rcedit.bkp and countries.mmdb are now under `/res/` at the project root.
    let res_dir = project_root.join("res"); // This is the new source directory

    let rcedit_source_bkp_path = res_dir.join("rcedit.bkp");
    // *** CORRECTED: Source file name is now 'countries.mmdb' (lowercase) ***
    let countries_mmdb_source_path = res_dir.join("countries.mmdb");

    // --- Define Destination Directory ---
    // Determine the target mode directory (e.g., target/debug/ or target/release/)
    let target_mode_dir = if cfg!(debug_assertions) {
        project_root.join("target").join("debug")
    } else {
        project_root.join("target").join("release")
    };

    // The new resources directory within target/debug/ or target/release/
    // This is the desired final location for these files in dev/standalone builds.
    let destination_resources_dir = target_mode_dir.join("resources");

    // --- Create Destination Directory ---
    println!(
        "Creating destination directory for dev/standalone assets: {:?}",
        destination_resources_dir
    );
    fs::create_dir_all(&destination_resources_dir)
        .expect(&format!(
            "Failed to create resources directory: {:?}",
            destination_resources_dir
        ));

    // --- Copy rcedit.bkp to target/{mode}/resources/rcedit.exe ---
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

    // --- Copy countries.mmdb to target/{mode}/resources/countries.mmdb ---
    // The destination name is already lowercase, and now the source is too.
    let countries_mmdb_destination_path = destination_resources_dir.join("countries.mmdb");
    println!(
        "Attempting to copy {:?} to {:?}",
        countries_mmdb_source_path, countries_mmdb_destination_path
    );
    if !countries_mmdb_source_path.exists() {
        // Updated error message to reflect new source path and name
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
    // Copy from countries_mmdb_source_path to countries_mmdb_destination_path
    fs::copy(&countries_mmdb_source_path, &countries_mmdb_destination_path)
        .expect("Failed to copy countries.mmdb to countries.mmdb");
    println!("Successfully copied countries.mmdb to countries.mmdb.");

    // --- Set Environment Variable for the main Rust application ---
    let dev_resources_path_str = destination_resources_dir
        .to_str()
        .expect("Failed to convert resources path to string")
        .to_string();
    println!("cargo:rustc-env=DEV_RESOURCES_PATH={}", dev_resources_path_str);

    // Tell Cargo to re-run this build script if source files change
    println!("cargo:rerun-if-changed={}", rcedit_source_bkp_path.display());
    println!("cargo:rerun-if-changed={}", countries_mmdb_source_path.display());
}

fn main() {
    restore_assets_for_dev_and_standalone();
    tauri_build::build();
}