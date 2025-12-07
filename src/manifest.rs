//! Parse and validate molt.toml manifest files.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Root manifest structure for molt.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct MoltManifest {
    /// Package metadata
    pub package: PackageConfig,
    /// Model definitions
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
    /// Target language configurations
    #[serde(default)]
    pub targets: HashMap<String, TargetConfig>,
}

/// Package metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct PackageConfig {
    /// Package name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Package description
    #[serde(default)]
    pub description: String,
    /// Author information
    #[serde(default)]
    pub authors: Vec<String>,
    /// License identifier
    #[serde(default)]
    pub license: Option<String>,
    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
}

/// Configuration for a single model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    /// Path to WIT file (relative to manifest)
    pub wit: PathBuf,
    /// World name in the WIT file
    pub world: String,
}

/// Configuration for a target language.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetConfig {
    /// Package name for the target language
    #[serde(default)]
    pub package_name: Option<String>,
    /// Crate name (Rust-specific)
    #[serde(default)]
    pub crate_name: Option<String>,
    /// Output directory (relative to manifest), defaults to "dist/{target}"
    #[serde(default)]
    pub output_dir: Option<PathBuf>,
    /// Additional target-specific options
    #[serde(default)]
    pub options: HashMap<String, toml::Value>,
}

impl MoltManifest {
    /// Load a manifest from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest file: {:?}", path))?;
        Self::parse(&content)
    }

    /// Parse a manifest from TOML string content.
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: MoltManifest =
            toml::from_str(content).with_context(|| "Failed to parse TOML manifest")?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest structure.
    fn validate(&self) -> Result<()> {
        if self.package.name.is_empty() {
            anyhow::bail!("Package name cannot be empty");
        }
        if self.package.version.is_empty() {
            anyhow::bail!("Package version cannot be empty");
        }

        for (name, model) in &self.models {
            if name.is_empty() {
                anyhow::bail!("Model name cannot be empty");
            }
            if model.world.is_empty() {
                anyhow::bail!("Model '{}' must specify a world", name);
            }
        }

        // Validate target-specific options if output_dir is specified
        for (_name, target) in &self.targets {
            if let Some(ref dir) = target.output_dir {
                if dir.as_os_str().is_empty() {
                    // Empty string output_dir is treated as None (use default)
                }
            }
        }

        Ok(())
    }

    /// Get the effective package name for a target.
    pub fn target_package_name(&self, target: &str) -> String {
        self.targets
            .get(target)
            .and_then(|t| t.package_name.clone().or_else(|| t.crate_name.clone()))
            .unwrap_or_else(|| self.package.name.replace('-', "_"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let content = r#"
[package]
name = "test-models"
version = "0.1.0"
"#;
        let manifest = MoltManifest::parse(content).unwrap();
        assert_eq!(manifest.package.name, "test-models");
        assert_eq!(manifest.package.version, "0.1.0");
    }

    #[test]
    fn test_parse_full_manifest() {
        let content = r#"
[package]
name = "physics-models"
version = "0.1.0"
description = "Aerodynamic and hydrodynamic models"

[models]
drag = { wit = "wit/drag.wit", world = "drag-model" }
hull = { wit = "wit/hull.wit", world = "hull-model" }

[targets.python]
package_name = "physics_models"
output_dir = "dist/python"

[targets.rust]
crate_name = "physics-models"
output_dir = "dist/rust"
"#;
        let manifest = MoltManifest::parse(content).unwrap();
        assert_eq!(manifest.package.name, "physics-models");
        assert_eq!(manifest.models.len(), 2);
        assert!(manifest.models.contains_key("drag"));
        assert!(manifest.models.contains_key("hull"));
        assert_eq!(manifest.targets.len(), 2);
    }

    #[test]
    fn test_empty_package_name() {
        let content = r#"
[package]
name = ""
version = "0.1.0"
"#;
        let result = MoltManifest::parse(content);
        assert!(result.is_err());
    }
}
