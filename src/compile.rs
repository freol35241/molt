//! WASM compilation via cargo-component.

use crate::manifest::MoltManifest;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Compile the project to WASM using cargo-component.
pub fn compile_to_wasm(project_dir: &Path) -> Result<HashMap<String, Vec<u8>>> {
    // Check for cargo-component
    let status = Command::new("cargo")
        .args(["component", "--version"])
        .current_dir(project_dir)
        .output();

    match status {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Using {}", version.trim());
        }
        _ => {
            anyhow::bail!(
                "cargo-component not found. Install it with:\n\
                 cargo install cargo-component"
            );
        }
    }

    // Run cargo component build
    println!("  Running cargo component build --release...");
    let output = Command::new("cargo")
        .args(["component", "build", "--release"])
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to execute cargo component build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "cargo component build failed:\n{}\n{}",
            stdout,
            stderr
        );
    }

    // Find the generated WASM file
    let target_dir = project_dir.join("target/wasm32-wasip1/release");

    // Also try wasm32-unknown-unknown target for older cargo-component versions
    let target_dir = if target_dir.exists() {
        target_dir
    } else {
        let alt_target = project_dir.join("target/wasm32-unknown-unknown/release");
        if alt_target.exists() {
            alt_target
        } else {
            // Try looking in the wasm32-wasip2 target (newer cargo-component)
            let wasip2_target = project_dir.join("target/wasm32-wasip2/release");
            if wasip2_target.exists() {
                wasip2_target
            } else {
                anyhow::bail!(
                    "Could not find WASM target directory. Tried:\n\
                     - {:?}\n\
                     - {:?}\n\
                     - {:?}",
                    project_dir.join("target/wasm32-wasip1/release"),
                    project_dir.join("target/wasm32-unknown-unknown/release"),
                    project_dir.join("target/wasm32-wasip2/release")
                );
            }
        }
    };

    // Get the package name from Cargo.toml to find the WASM file
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {:?}", cargo_toml_path))?;
    let cargo_manifest: toml::Table = toml::from_str(&cargo_toml)
        .with_context(|| "Failed to parse Cargo.toml")?;

    let package_name = cargo_manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))?;

    // WASM filename uses underscores
    let wasm_name = package_name.replace('-', "_");
    let wasm_path = target_dir.join(format!("{}.wasm", wasm_name));

    if !wasm_path.exists() {
        // Try looking for any .wasm file
        let entries: Vec<_> = std::fs::read_dir(&target_dir)
            .with_context(|| format!("Failed to read {:?}", target_dir))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "wasm"))
            .collect();

        if entries.is_empty() {
            anyhow::bail!(
                "No WASM files found in {:?}. Expected {:?}",
                target_dir,
                wasm_path
            );
        }

        // Use the first .wasm file found
        let wasm_path = entries[0].path();
        println!("  Found WASM: {:?}", wasm_path);
        let bytes = std::fs::read(&wasm_path)
            .with_context(|| format!("Failed to read {:?}", wasm_path))?;

        let mut result = HashMap::new();
        result.insert("main".to_string(), bytes);
        return Ok(result);
    }

    println!("  Built: {:?}", wasm_path);
    let bytes = std::fs::read(&wasm_path)
        .with_context(|| format!("Failed to read {:?}", wasm_path))?;

    // For MVP, we use a single WASM module for all models
    let mut result = HashMap::new();
    result.insert("main".to_string(), bytes);

    Ok(result)
}

/// Find existing WASM file (for --skip-compile mode).
pub fn find_existing_wasm(project_dir: &Path, _manifest: &MoltManifest) -> Result<HashMap<String, Vec<u8>>> {
    // Look for WASM file in standard locations
    let possible_targets = [
        "target/wasm32-wasip1/release",
        "target/wasm32-wasip2/release",
        "target/wasm32-unknown-unknown/release",
    ];

    for target in &possible_targets {
        let target_dir = project_dir.join(target);
        if !target_dir.exists() {
            continue;
        }

        let entries: Vec<_> = std::fs::read_dir(&target_dir)
            .with_context(|| format!("Failed to read {:?}", target_dir))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "wasm"))
            .collect();

        if let Some(entry) = entries.first() {
            let wasm_path = entry.path();
            println!("  Found existing WASM: {:?}", wasm_path);
            let bytes = std::fs::read(&wasm_path)
                .with_context(|| format!("Failed to read {:?}", wasm_path))?;

            let mut result = HashMap::new();
            result.insert("main".to_string(), bytes);
            return Ok(result);
        }
    }

    anyhow::bail!(
        "No existing WASM file found. Run `molt build` without --skip-compile first."
    );
}
