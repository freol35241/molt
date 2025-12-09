//! WASM compilation via cargo-component.

use crate::manifest::MoltManifest;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Find the cargo binary path.
///
/// Uses a cross-platform approach:
/// 1. Try `which`/`where` command to find cargo in PATH
/// 2. Check standard installation locations based on user's home directory
/// 3. Fall back to bare "cargo" and let the OS resolve it
fn find_cargo() -> Result<std::path::PathBuf> {
    // First, try to find cargo via PATH using which (Unix) or where (Windows)
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    if let Ok(output) = Command::new(which_cmd).arg("cargo").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !path.is_empty() {
                let cargo_path = std::path::PathBuf::from(&path);
                if cargo_path.exists() {
                    return Ok(cargo_path);
                }
            }
        }
    }

    // Try standard installation locations based on user's home directory
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".cargo").join("bin").join(cargo_binary_name()),
            home.join(".rustup")
                .join("toolchains")
                .join("stable-x86_64-unknown-linux-gnu")
                .join("bin")
                .join(cargo_binary_name()),
            home.join(".rustup")
                .join("toolchains")
                .join("stable-aarch64-unknown-linux-gnu")
                .join("bin")
                .join(cargo_binary_name()),
            home.join(".rustup")
                .join("toolchains")
                .join("stable-x86_64-apple-darwin")
                .join("bin")
                .join(cargo_binary_name()),
            home.join(".rustup")
                .join("toolchains")
                .join("stable-aarch64-apple-darwin")
                .join("bin")
                .join(cargo_binary_name()),
        ];

        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }
    }

    // Try system-wide installation paths
    let system_paths = if cfg!(windows) {
        vec![std::path::PathBuf::from("C:\\Program Files\\Rust\\bin\\cargo.exe")]
    } else {
        vec![
            std::path::PathBuf::from("/usr/local/cargo/bin/cargo"),
            std::path::PathBuf::from("/usr/local/bin/cargo"),
            std::path::PathBuf::from("/usr/bin/cargo"),
        ]
    };

    for path in system_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    // Fall back to just "cargo" and let PATH handle it
    Ok(std::path::PathBuf::from(cargo_binary_name()))
}

/// Get the cargo binary name for the current platform.
fn cargo_binary_name() -> &'static str {
    if cfg!(windows) {
        "cargo.exe"
    } else {
        "cargo"
    }
}

/// Get environment with cargo bin in PATH.
///
/// Ensures the user's cargo installation directories are available
/// in PATH and sets RUSTUP_HOME/CARGO_HOME if not already set.
fn get_cargo_env() -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = std::env::vars().collect();

    // Get home directory - if unavailable, just return current env
    let Some(home) = dirs::home_dir() else {
        return env;
    };

    // Ensure ~/.cargo/bin is in PATH
    let cargo_bin = home.join(".cargo").join("bin");
    if cargo_bin.exists() {
        let path_sep = if cfg!(windows) { ";" } else { ":" };
        let path = env
            .iter()
            .find(|(k, _)| k == "PATH")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        if !path.contains(cargo_bin.to_string_lossy().as_ref()) {
            let new_path = format!("{}{}{}", cargo_bin.display(), path_sep, path);
            env.retain(|(k, _)| k != "PATH");
            env.push(("PATH".to_string(), new_path));
        }
    }

    // Ensure RUSTUP_HOME is set if the directory exists
    if !env.iter().any(|(k, _)| k == "RUSTUP_HOME") {
        let rustup_home = home.join(".rustup");
        if rustup_home.exists() {
            env.push((
                "RUSTUP_HOME".to_string(),
                rustup_home.to_string_lossy().to_string(),
            ));
        }
    }

    // Ensure CARGO_HOME is set if the directory exists
    if !env.iter().any(|(k, _)| k == "CARGO_HOME") {
        let cargo_home = home.join(".cargo");
        if cargo_home.exists() {
            env.push((
                "CARGO_HOME".to_string(),
                cargo_home.to_string_lossy().to_string(),
            ));
        }
    }

    env
}

/// Compile the project to WASM using cargo-component.
pub fn compile_to_wasm(project_dir: &Path) -> Result<HashMap<String, Vec<u8>>> {
    let cargo = find_cargo()?;
    let env = get_cargo_env();

    // Check for cargo-component
    let status = Command::new(&cargo)
        .args(["component", "--version"])
        .current_dir(project_dir)
        .envs(env.clone())
        .output();

    match status {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Using {}", version.trim());
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "cargo-component not found. Install it with:\n\
                 cargo install cargo-component\n\
                 Exit code: {:?}\n\
                 stdout: {}\n\
                 stderr: {}",
                output.status.code(),
                stdout,
                stderr
            );
        }
        Err(e) => {
            anyhow::bail!(
                "cargo-component not found. Install it with:\n\
                 cargo install cargo-component\n\
                 Error: {}",
                e
            );
        }
    }

    // Run cargo component build
    println!("  Running cargo component build --release...");
    let output = Command::new(&cargo)
        .args(["component", "build", "--release"])
        .current_dir(project_dir)
        .envs(env)
        .output()
        .with_context(|| "Failed to execute cargo component build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!("cargo component build failed:\n{}\n{}", stdout, stderr);
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
    let cargo_manifest: toml::Table =
        toml::from_str(&cargo_toml).with_context(|| "Failed to parse Cargo.toml")?;

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
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wasm"))
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
        let bytes =
            std::fs::read(&wasm_path).with_context(|| format!("Failed to read {:?}", wasm_path))?;

        let mut result = HashMap::new();
        result.insert("main".to_string(), bytes);
        return Ok(result);
    }

    println!("  Built: {:?}", wasm_path);
    let bytes =
        std::fs::read(&wasm_path).with_context(|| format!("Failed to read {:?}", wasm_path))?;

    // For MVP, we use a single WASM module for all models
    let mut result = HashMap::new();
    result.insert("main".to_string(), bytes);

    Ok(result)
}

/// Find existing WASM file (for --skip-compile mode).
///
/// Uses the manifest package name to locate the expected WASM file,
/// falling back to any .wasm file if the expected one isn't found.
pub fn find_existing_wasm(
    project_dir: &Path,
    manifest: &MoltManifest,
) -> Result<HashMap<String, Vec<u8>>> {
    // Expected WASM filename based on package name (using underscores)
    let expected_wasm_name = format!("{}.wasm", manifest.package.name.replace('-', "_"));

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

        // First, try to find the expected WASM file by name
        let expected_path = target_dir.join(&expected_wasm_name);
        if expected_path.exists() {
            println!("  Found existing WASM: {:?}", expected_path);
            let bytes = std::fs::read(&expected_path)
                .with_context(|| format!("Failed to read {:?}", expected_path))?;

            let mut result = HashMap::new();
            result.insert("main".to_string(), bytes);
            return Ok(result);
        }

        // Fall back to any .wasm file in the directory
        let entries: Vec<_> = std::fs::read_dir(&target_dir)
            .with_context(|| format!("Failed to read {:?}", target_dir))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wasm"))
            .collect();

        if let Some(entry) = entries.first() {
            let wasm_path = entry.path();
            println!(
                "  Found existing WASM: {:?} (expected {})",
                wasm_path, expected_wasm_name
            );
            let bytes = std::fs::read(&wasm_path)
                .with_context(|| format!("Failed to read {:?}", wasm_path))?;

            let mut result = HashMap::new();
            result.insert("main".to_string(), bytes);
            return Ok(result);
        }
    }

    anyhow::bail!(
        "No existing WASM file found for package '{}'. \
         Expected '{}' in one of: {:?}. \
         Run `molt build` without --skip-compile first.",
        manifest.package.name,
        expected_wasm_name,
        possible_targets
    );
}
