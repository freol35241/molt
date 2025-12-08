//! molt CLI - Model Once, Load Trivially
//!
//! Build-time toolkit for multi-language model packages.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use molt::codegen;
use molt::compile;
use molt::glue;
use molt::manifest::MoltManifest;
use molt::wit;

#[derive(Parser)]
#[command(name = "molt")]
#[command(author, version, about = "Model Once, Load Trivially", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build model packages (runs generate, compile, pack)
    Build {
        /// Path to molt.toml manifest (defaults to current directory)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Target language to build (if omitted, builds all targets)
        #[arg(short, long)]
        target: Option<String>,

        /// Skip WASM compilation (use existing .wasm file)
        #[arg(long)]
        skip_compile: bool,
    },

    /// Generate glue code in molt-gen/
    Generate {
        /// Path to molt.toml manifest (defaults to current directory)
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },

    /// Generate output packages from compiled WASM
    Pack {
        /// Path to molt.toml manifest (defaults to current directory)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Target language to pack (if omitted, packs all targets)
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Initialize a new molt project
    Init {
        /// Project name
        name: String,

        /// Directory to create project in (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Validate molt.toml and WIT files without building
    Check {
        /// Path to molt.toml manifest
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            manifest,
            target,
            skip_compile,
        } => cmd_build(manifest, target, skip_compile),
        Commands::Generate { manifest } => cmd_generate(manifest),
        Commands::Pack { manifest, target } => cmd_pack(manifest, target),
        Commands::Init { name, path } => cmd_init(name, path),
        Commands::Check { manifest } => cmd_check(manifest),
    }
}

fn cmd_build(
    manifest_path: Option<PathBuf>,
    target: Option<String>,
    skip_compile: bool,
) -> Result<()> {
    let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("molt.toml"));
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    println!("Loading manifest from {:?}", manifest_path);
    let manifest = MoltManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load manifest from {:?}", manifest_path))?;

    // Parse WIT files and extract model interfaces
    println!("Parsing WIT interfaces...");
    let models = parse_models(&manifest, &project_dir)?;

    // Step 1: Generate glue code
    println!("Generating glue code...");
    let glue_content = glue::generate_glue(&manifest, &models, &project_dir)?;
    glue::write_glue(&project_dir, &glue_content)?;
    println!("  Generated molt-gen/lib.rs");

    // Step 2: Compile to WASM
    let wasm_bytes = if skip_compile {
        println!("Skipping WASM compilation (--skip-compile)");
        compile::find_existing_wasm(&project_dir, &manifest)?
    } else {
        println!("Compiling to WASM...");
        compile::compile_to_wasm(&project_dir)?
    };

    // Step 3: Generate packages
    generate_packages(&manifest, &models, &wasm_bytes, &project_dir, target)?;

    println!("Build complete!");
    Ok(())
}

fn cmd_generate(manifest_path: Option<PathBuf>) -> Result<()> {
    let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("molt.toml"));
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    println!("Loading manifest from {:?}", manifest_path);
    let manifest = MoltManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load manifest from {:?}", manifest_path))?;

    // Parse WIT files and extract model interfaces
    println!("Parsing WIT interfaces...");
    let models = parse_models(&manifest, &project_dir)?;

    // Generate glue code
    println!("Generating glue code...");
    let glue_content = glue::generate_glue(&manifest, &models, &project_dir)?;
    glue::write_glue(&project_dir, &glue_content)?;
    println!("  Generated molt-gen/lib.rs");

    println!("Generate complete!");
    Ok(())
}

fn cmd_pack(manifest_path: Option<PathBuf>, target: Option<String>) -> Result<()> {
    let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("molt.toml"));
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    println!("Loading manifest from {:?}", manifest_path);
    let manifest = MoltManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load manifest from {:?}", manifest_path))?;

    // Parse WIT files and extract model interfaces
    println!("Parsing WIT interfaces...");
    let models = parse_models(&manifest, &project_dir)?;

    // Find existing WASM
    println!("Looking for compiled WASM...");
    let wasm_bytes = compile::find_existing_wasm(&project_dir, &manifest)?;

    // Generate packages
    generate_packages(&manifest, &models, &wasm_bytes, &project_dir, target)?;

    println!("Pack complete!");
    Ok(())
}

/// Parse all model interfaces from WIT files.
fn parse_models(manifest: &MoltManifest, project_dir: &Path) -> Result<Vec<molt::ModelInterface>> {
    let mut models = Vec::new();
    for (name, model_config) in &manifest.models {
        let wit_path = project_dir.join(&model_config.wit);
        println!("  Parsing {:?} for model '{}'", wit_path, name);
        let model_interface = wit::parse_wit_file(&wit_path, &model_config.world, name)
            .with_context(|| format!("Failed to parse WIT file {:?}", wit_path))?;
        models.push(model_interface);
    }
    Ok(models)
}

/// Generate packages for specified targets.
fn generate_packages(
    manifest: &MoltManifest,
    models: &[molt::ModelInterface],
    wasm_bytes: &std::collections::HashMap<String, Vec<u8>>,
    project_dir: &Path,
    target: Option<String>,
) -> Result<()> {
    let targets_to_build: Vec<_> = if let Some(ref t) = target {
        manifest
            .targets
            .iter()
            .filter(|(name, _)| name.as_str() == t)
            .collect()
    } else {
        manifest.targets.iter().collect()
    };

    if targets_to_build.is_empty() {
        if let Some(t) = target {
            anyhow::bail!("Target '{}' not found in manifest", t);
        }
        println!("No targets configured in manifest");
        return Ok(());
    }

    for (target_name, target_config) in targets_to_build {
        println!("Generating {} package...", target_name);
        let output_dir = target_config
            .output_dir
            .as_ref()
            .map(|p| project_dir.join(p))
            .unwrap_or_else(|| project_dir.join(format!("dist/{}", target_name)));

        match target_name.as_str() {
            "python" => {
                codegen::python::generate_python_package(
                    manifest,
                    models,
                    wasm_bytes,
                    &output_dir,
                    target_config,
                )?;
            }
            "rust" => {
                println!("  Rust target not yet implemented (post-MVP)");
            }
            "javascript" | "js" => {
                println!("  JavaScript target not yet implemented (post-MVP)");
            }
            other => {
                println!("  Unknown target: {}", other);
            }
        }
    }

    Ok(())
}

fn cmd_init(name: String, path: Option<PathBuf>) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| PathBuf::from(&name));

    if project_dir.exists() {
        anyhow::bail!("Directory {:?} already exists", project_dir);
    }

    println!("Creating new molt project '{}'...", name);

    std::fs::create_dir_all(&project_dir)?;
    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::create_dir_all(project_dir.join("wit"))?;

    let ns = name.replace('-', "");
    let name_snake = name.replace('-', "_");

    // Create molt.toml with new fields
    let molt_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
description = "A molt model package"

[models.example]
wit = "wit/example.wit"
world = "example-model"
source = "src/example.rs"
struct = "ExampleModel"
outputs = "ExampleOutputs"

[targets.python]
package_name = "{name_snake}"
output_dir = "dist/python"
"#,
        name = name,
        name_snake = name_snake
    );
    std::fs::write(project_dir.join("molt.toml"), molt_toml)?;

    // Create example WIT with predict method and result type
    let wit_content = format!(
        r#"package {ns}:models;

interface example {{
    record params {{
        coefficient: float64,
    }}

    record inputs {{
        value: float64,
    }}

    record outputs {{
        scaled: float64,
    }}

    resource model {{
        constructor(p: params);
        predict: func(i: inputs) -> result<outputs, string>;
    }}
}}

world example-model {{
    export example;
}}
"#,
        ns = ns
    );
    std::fs::write(project_dir.join("wit/example.wit"), wit_content)?;

    // Create Cargo.toml pointing to molt-gen/lib.rs
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
path = "molt-gen/lib.rs"
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.36"

[package.metadata.component]
package = "{ns}:models"

[package.metadata.component.target]
path = "wit"
"#,
        name = name,
        ns = ns
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create pure Rust implementation (no wit-bindgen macros)
    let example_rs = r#"//! Example model implementation.
//!
//! This is pure Rust - no wit-bindgen macros needed.
//! molt generates the glue code in molt-gen/lib.rs.

pub struct ExampleModel {
    coefficient: f64,
}

impl ExampleModel {
    /// Create a new example model.
    pub fn new(coefficient: f64) -> Result<Self, String> {
        if coefficient == 0.0 {
            return Err("coefficient cannot be zero".to_string());
        }
        Ok(Self { coefficient })
    }

    /// Predict the scaled value.
    pub fn predict(&self, value: f64) -> Result<ExampleOutputs, String> {
        Ok(ExampleOutputs {
            scaled: value * self.coefficient,
        })
    }
}

pub struct ExampleOutputs {
    pub scaled: f64,
}
"#;
    std::fs::write(project_dir.join("src/example.rs"), example_rs)?;

    // Create .gitignore
    let gitignore = r#"# Generated by molt
molt-gen/
dist/
target/
"#;
    std::fs::write(project_dir.join(".gitignore"), gitignore)?;

    println!("Created project at {:?}", project_dir);
    println!("\nProject structure:");
    println!("  src/example.rs   - Your pure Rust implementation");
    println!("  wit/example.wit  - WIT interface definition");
    println!("  molt.toml        - MOLT configuration");
    println!("  molt-gen/        - Generated glue code (git-ignored)");
    println!("\nNext steps:");
    println!("  cd {}", project_dir.display());
    println!("  molt build");

    Ok(())
}

fn cmd_check(manifest_path: Option<PathBuf>) -> Result<()> {
    let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("molt.toml"));
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    println!("Checking {:?}...", manifest_path);

    let manifest = MoltManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load manifest from {:?}", manifest_path))?;
    println!("  Manifest OK: {}", manifest.package.name);

    for (name, model_config) in &manifest.models {
        let wit_path = project_dir.join(&model_config.wit);
        print!("  Checking model '{}' ({:?})... ", name, wit_path);
        let model = wit::parse_wit_file(&wit_path, &model_config.world, name)
            .with_context(|| format!("Failed to parse WIT file {:?}", wit_path))?;
        println!("OK");
        println!(
            "    params: {:?}",
            model
                .params
                .fields
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
        println!(
            "    inputs: {:?}",
            model
                .inputs
                .fields
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
        println!(
            "    outputs: {:?}",
            model
                .outputs
                .fields
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    println!("\nAll checks passed!");
    Ok(())
}
