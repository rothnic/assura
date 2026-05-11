//! Command-line interface modules and public CLI re-exports.
pub mod args;
pub mod check;
pub mod commands;
pub mod config;
pub mod hooks;
pub mod output;

pub use args::{Cli, Commands, ExitCode, HookCommands, OutputFormat};
pub use check::{run_structure_check, CheckError, StructureCheckReport, StructureViolation};
pub use commands::{check_command, init_command, status_command, watch_command};
pub use config::{CliConfig, ConfigDiscovery};
pub use hooks::{GitHooksManager, HookType};
pub use output::{OutputFormatter, ValidationReporter};
