//! molt - Model Once, Load Trivially
//!
//! A build-time toolkit that lets library maintainers write physical/mathematical
//! models once and distribute them as idiomatic, typed packages for multiple languages.

pub mod codegen;
pub mod compile;
pub mod glue;
pub mod interface;
pub mod manifest;
pub mod wit;

pub use interface::{FieldDef, ModelInterface, RecordDef, WitType};
pub use manifest::{ModelConfig, MoltManifest, PackageConfig, TargetConfig};
