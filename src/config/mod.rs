//! Configuration module for Assura
//!
//! The current authored configuration contract is `config::Config` loaded by
//! `ConfigLoader`. It parses, normalizes, and validates `structure` notation
//! for every supported runtime command.
//!
//! `ast::LegacyNotationConfig` and `types::LegacyPolicyConfig` remain only for
//! the named legacy notation and compatibility validation paths. They do not
//! describe the current `structure` language and must not be used by runtime
//! command handlers.

#[cfg(feature = "full-cli")]
pub mod ast;
// allow-reason: the public module name mirrors the existing configuration
// namespace used throughout the CLI before a pre-1.0 module split.
#[allow(clippy::module_inception)]
pub mod config;
#[cfg(feature = "full-cli")]
pub mod engine;
pub mod inheritance;
#[cfg(feature = "yaml-config")]
pub mod loader;
pub mod ls_compat;
#[cfg(test)]
mod ls_compat_tests;
#[cfg(feature = "full-cli")]
pub mod parser;
#[cfg(feature = "full-cli")]
pub mod preprocessor;
pub mod types;
#[cfg(feature = "full-cli")]
pub mod validator;

/// Legacy notation types retained for the LS-Lint compatibility adapter and
/// legacy validation tests.
#[cfg(feature = "full-cli")]
pub use ast::{Constraint, Context, LegacyNotationConfig, PolicyNode, Rule, ViolationEntry};
/// Current normalized structure configuration used by runtime commands.
pub use config::Config;
/// Current loader for authored structure notation.
#[cfg(feature = "yaml-config")]
pub use loader::ConfigLoader;
#[cfg(feature = "full-cli")]
pub use parser::{LegacyConfigParser, LegacyParseError};
#[cfg(feature = "full-cli")]
pub use preprocessor::YamlPreprocessor;

/// Load configuration from default locations
#[cfg(feature = "full-cli")]
pub fn load_config() -> Result<config::Config, Box<dyn std::error::Error>> {
    // Try .assura/config.yml
    let path = std::path::Path::new(".assura/config.yml");
    if path.exists() {
        return Ok(ConfigLoader::load(path)?);
    }

    // Try .assura/config.yaml
    let path = std::path::Path::new(".assura/config.yaml");
    if path.exists() {
        return Ok(ConfigLoader::load(path)?);
    }

    Err("No configuration file found".into())
}
