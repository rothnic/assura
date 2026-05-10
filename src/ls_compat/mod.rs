//! LS-Lint Compatibility Module
//!
//! Provides migration from LS-Lint to Assura format.

#[cfg(test)]
mod parity_tests;
pub mod parser;

pub use parser::{LsLintConfig, LsLintParseError, LsLintParser, MigrationReport, MigrationTool};
