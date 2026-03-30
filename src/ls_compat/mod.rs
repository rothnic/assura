//! LS-Lint Compatibility Module
//!
//! Provides migration from LS-Lint to Assura format.

pub mod parser;
#[cfg(test)]
mod parity_tests;

pub use parser::{LsLintParser, LsLintConfig, MigrationTool, MigrationReport, LsLintParseError};
