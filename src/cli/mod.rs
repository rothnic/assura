//! Command-line interface modules and public CLI re-exports.
#[cfg(feature = "full-cli")]
pub mod args;
pub mod check;
#[cfg(feature = "full-cli")]
pub mod commands;
pub mod config;
#[cfg(feature = "full-cli")]
pub mod hooks;
#[cfg(feature = "full-cli")]
pub mod init_support;
#[cfg(feature = "full-cli")]
pub mod output;
#[cfg(feature = "full-cli")]
pub mod performance_report;

#[cfg(feature = "full-cli")]
pub use args::{Cli, Commands, ExitCode, HookCommands, OutputFormat, PerformanceReportFormat};
#[cfg(all(feature = "yaml-config", feature = "json-output"))]
pub use check::run_structure_check_cached;
#[cfg(feature = "yaml-config")]
pub use check::{
    run_structure_check, run_structure_check_with_timings, run_structure_checks,
    PreparedStructureCheck,
};
pub use check::{
    run_structure_check_with_artifact, run_structure_check_with_config,
    run_structure_check_with_fast_artifact, run_structure_check_with_prechecked_fast_artifact,
    CheckError, CompiledStructureConfigArtifact, StructureCheckReport, StructureCheckTimings,
    StructureViolation,
};
#[cfg(feature = "full-cli")]
pub use commands::{
    check_command, info_command, init_command, migrate_command, status_command, watch_command,
};
#[cfg(feature = "full-cli")]
pub use config::CliConfig;
pub use config::ConfigDiscovery;
#[cfg(feature = "full-cli")]
pub use hooks::{GitHooksManager, HookType};
#[cfg(feature = "full-cli")]
pub use output::{OutputFormatter, ValidationReporter};
#[cfg(feature = "full-cli")]
pub use performance_report::{
    performance_report_command, PerformanceReport, PerformanceReportCommandOptions,
    PerformanceResultRow,
};
