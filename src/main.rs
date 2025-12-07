//! molt CLI - Model Once, Load Trivially
//!
//! Build-time toolkit for multi-language model packages.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use molt::codegen;
use molt::compile;
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
    /// Build model packages for target languages
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
    let mut models = Vec::new();
    for (name, model_config) in &manifest.models {
        let wit_path = project_dir.join(&model_config.wit);
        println!("  Parsing {:?} for model '{}'", wit_path, name);
        let model_interface = wit::parse_wit_file(&wit_path, &model_config.world, name)
            .with_context(|| format!("Failed to parse WIT file {:?}", wit_path))?;
        models.push(model_interface);
    }

    // Compile to WASM
    let wasm_bytes = if skip_compile {
        println!("Skipping WASM compilation (--skip-compile)");
        // Try to find existing WASM file
        compile::find_existing_wasm(&project_dir, &manifest)?
    } else {
        println!("Compiling to WASM...");
        compile::compile_to_wasm(&project_dir)?
    };

    // Generate packages for each target
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
        let output_dir = project_dir.join(&target_config.output_dir);

        match target_name.as_str() {
            "python" => {
                codegen::python::generate_python_package(
                    &manifest,
                    &models,
                    &wasm_bytes,
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

    println!("Build complete!");
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

    // Create molt.toml
    let molt_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
description = "A molt model package"

[models]
example = {{ wit = "wit/example.wit", world = "example-model" }}

[targets.python]
package_name = "{name_snake}"
output_dir = "dist/python"
"#,
        name = name,
        name_snake = name.replace('-', "_")
    );
    std::fs::write(project_dir.join("molt.toml"), molt_toml)?;

    // Create example WIT
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
        result: float64,
    }}

    resource model {{
        constructor(p: params);
        step: func(i: inputs) -> outputs;
    }}
}}

world example-model {{
    export example;
}}
"#,
        ns = name.replace('-', "")
    );
    std::fs::write(project_dir.join("wit/example.wit"), wit_content)?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.36"

[package.metadata.component]
package = "{ns}:models"

[package.metadata.component.target]
path = "wit"
"#,
        name = name,
        ns = name.replace('-', "")
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create lib.rs
    let lib_rs = r#"wit_bindgen::generate!({
    world: "example-model",
});

use exports::example::{Guest, GuestModel, Inputs, Outputs, Params};

pub struct ExampleModel {
    params: Params,
}

impl GuestModel for ExampleModel {
    fn new(p: Params) -> Self {
        Self { params: p }
    }

    fn step(&self, i: Inputs) -> Outputs {
        Outputs {
            result: i.value * self.params.coefficient,
        }
    }
}

pub struct Example;

impl Guest for Example {
    type Model = ExampleModel;
}

export!(Example);
"#;
    std::fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    println!("Created project at {:?}", project_dir);
    println!("\nNext steps:");
    println!("  cd {}", project_dir.display());
    println!("  # Edit wit/example.wit and src/lib.rs");
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
