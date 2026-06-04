//! Legacy LS-Lint compatibility module.
//!
//! This module is retained for internal regression tests only. Runtime
//! migration behavior and migration reports are authoritative in
//! `crate::config::ls_compat`.

#[cfg(test)]
mod parity_tests;
pub mod parser;

pub use parser::{LsLintParser, MigrationTool};
