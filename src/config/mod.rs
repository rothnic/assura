//! Configuration module for Assura
//!
//! This module handles parsing and representation of Assura configuration files.
//! Follows Constitution principles: structure-first, valid YAML/JSON.

pub mod ast;
pub mod config;
pub mod engine;
pub mod inheritance;
pub mod loader;
pub mod ls_compat;
pub mod parser;
pub mod preprocessor;
pub mod types;
pub mod validator;

// Re-export main types
pub use ast::{Config, Constraint, Context, PolicyNode, Rule, ViolationEntry};
pub use parser::{ConfigParser, ParseError};
pub use preprocessor::YamlPreprocessor;

/// Load configuration from default locations
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
