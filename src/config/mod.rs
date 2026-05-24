//! Configuration module for Assura
//!
//! This module handles parsing and representation of Assura configuration files.
//! Follows Constitution principles: structure-first, valid YAML/JSON.

#[cfg(feature = "full-cli")]
pub mod ast;
#[allow(clippy::module_inception)]
pub mod config;
#[cfg(feature = "full-cli")]
pub mod engine;
pub mod inheritance;
#[cfg(feature = "yaml-config")]
pub mod loader;
pub mod ls_compat;
#[cfg(feature = "full-cli")]
pub mod parser;
#[cfg(feature = "full-cli")]
pub mod preprocessor;
#[cfg(feature = "full-cli")]
pub mod types;
#[cfg(feature = "full-cli")]
pub mod validator;

// Re-export main types
#[cfg(feature = "full-cli")]
pub use ast::{Config, Constraint, Context, PolicyNode, Rule, ViolationEntry};
#[cfg(feature = "full-cli")]
pub use parser::{ConfigParser, ParseError};
#[cfg(feature = "full-cli")]
pub use preprocessor::YamlPreprocessor;

/// Load configuration from default locations
#[cfg(feature = "full-cli")]
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Try .assura/config.yml
    let path = std::path::Path::new(".assura/config.yml");
    if path.exists() {
        return Ok(ConfigParser::parse_file(path)?);
    }

    // Try .assura/config.yaml
    let path = std::path::Path::new(".assura/config.yaml");
    if path.exists() {
        return Ok(ConfigParser::parse_file(path)?);
    }

    Err("No configuration file found".into())
}
