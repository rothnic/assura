//! CLI commands for local daemon process and daemon-ready project state.

#[path = "daemon_lifecycle.rs"]
mod lifecycle;
#[path = "daemon_management.rs"]
mod management;
#[path = "daemon_process.rs"]
mod process;
#[path = "daemon_references.rs"]
mod references;
#[path = "daemon_render.rs"]
mod render;
#[path = "daemon_text.rs"]
mod text;
#[path = "daemon_transport.rs"]
mod transport;

use super::{ExitCode, OutputFormat};
use crate::cli::check::StructureCheckReport;
use crate::daemon::{
    DaemonAffectedReferences, DaemonCoreError, DaemonHealth, DaemonMovedTargetReferences,
    LocalDaemonCore,
};
use clap::Subcommand;
use lifecycle::{
    daemon_logs_output, daemon_restart_output, daemon_start_output, daemon_stop_output,
};
use management::{daemon_doctor_output, daemon_status_output, health_for_path};
use process::{request_check_path, serve_daemon};
use references::{reference_request, DaemonReferenceRequest};
pub(crate) use render::DaemonTextRender;
use render::{
    exit_code_from_i32, render_error, render_load_error, render_raw_value, render_success,
    render_success_with_exit, DaemonCommandError, DaemonCommandOutcome,
};
use serde::Serialize;
use std::path::PathBuf;

/// Daemon-ready probe commands.
#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Report daemon management status for a project.
    Status {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Start a local daemon process for a project.
    Start {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Stop the local daemon process for a project.
    Stop {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Restart the local daemon process for a project.
    Restart {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Report daemon management diagnostics and remediation commands.
    Doctor {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Show daemon runtime log lines for a project.
    Logs {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Maximum log lines to return from the end of the file.
        #[arg(long, default_value_t = 100)]
        tail: usize,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Report daemon-ready health metadata for a project.
    Health {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Validate one changed path against prepared structure state.
    CheckPath {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Changed file or directory path.
        #[arg(long)]
        changed: PathBuf,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Report bounded repository-reference context for a changed path.
    References {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

        /// Changed source path for outbound references.
        #[arg(long)]
        source: Option<PathBuf>,

        /// Changed target path for inbound references.
        #[arg(long)]
        target: Option<PathBuf>,

        /// Previous target path for move feedback.
        #[arg(long)]
        moved_target: Option<PathBuf>,

        /// New target path for move feedback.
        #[arg(long, requires = "moved_target")]
        new_target: Option<PathBuf>,

        /// Maximum references to return.
        #[arg(long, default_value_t = 20)]
        limit: usize,

        /// Output format.
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Run the local daemon server process.
    #[command(hide = true)]
    Serve {
        /// Project root directory.
        path: PathBuf,

        /// IPC listen address.
        #[arg(long)]
        listen: String,
    },
}

/// Run a daemon-ready probe command.
pub async fn daemon_command(command: DaemonCommands, config: Option<PathBuf>) -> ExitCode {
    match run_daemon_command(command, config) {
        Ok(outcome) => {
            if !outcome.rendered.is_empty() {
                println!("{}", outcome.rendered);
            }
            outcome.exit_code
        }
        Err(error) => {
            eprintln!("Error: {}", error.message);
            error.exit_code
        }
    }
}

fn run_daemon_command(
    command: DaemonCommands,
    config: Option<PathBuf>,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
    let format = command.format();
    let path = command.path();
    if let DaemonCommands::Serve { path, listen } = command {
        return match serve_daemon(path, config, listen) {
            Ok(()) => Ok(DaemonCommandOutcome {
                rendered: String::new(),
                exit_code: ExitCode::Success,
            }),
            Err(message) => Err(DaemonCommandError {
                message,
                exit_code: ExitCode::RuntimeError,
            }),
        };
    }
    let reference_request = match &command {
        DaemonCommands::References {
            source,
            target,
            moved_target,
            new_target,
            ..
        } => Some(
            reference_request(
                source.clone(),
                target.clone(),
                moved_target.clone(),
                new_target.clone(),
            )
            .map_err(|message| DaemonCommandError {
                message,
                exit_code: ExitCode::ConfigurationError,
            })?,
        ),
        _ => None,
    };
    match &command {
        DaemonCommands::Status { .. } => {
            let (health, _) = health_for_path(path, config);
            return render_success(daemon_status_output(health), format);
        }
        DaemonCommands::Start { .. } => {
            let (health, loaded) = health_for_path(path, config);
            let output = daemon_start_output(health, loaded);
            let exit_code = if loaded && output.succeeded() {
                ExitCode::Success
            } else {
                ExitCode::RuntimeError
            };
            return render_success_with_exit(output, format, exit_code);
        }
        DaemonCommands::Stop { .. } => {
            let (health, _) = health_for_path(path, config);
            return render_success(daemon_stop_output(health), format);
        }
        DaemonCommands::Restart { .. } => {
            let (health, loaded) = health_for_path(path, config);
            let output = daemon_restart_output(health, loaded);
            let exit_code = if loaded && output.succeeded() {
                ExitCode::Success
            } else {
                ExitCode::RuntimeError
            };
            return render_success_with_exit(output, format, exit_code);
        }
        DaemonCommands::Doctor { .. } => {
            let (health, loaded) = health_for_path(path, config);
            let output = daemon_doctor_output(health, loaded);
            return render_success_with_exit(
                output,
                format,
                if loaded {
                    ExitCode::Success
                } else {
                    ExitCode::RuntimeError
                },
            );
        }
        DaemonCommands::Logs { tail, .. } => {
            let (health, _) = health_for_path(path, config);
            return render_success(daemon_logs_output(health, *tail), format);
        }
        _ => {}
    }
    let mut core = match LocalDaemonCore::load(path, config) {
        Ok(core) => core,
        Err(error) => return render_load_error(error, format, command.path()),
    };

    match command {
        DaemonCommands::Health { .. } => render_success(core.health(), format),
        DaemonCommands::CheckPath { changed, .. } => {
            if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
                let runtime = lifecycle::runtime_status_for_health(&core.health());
                if matches!(
                    runtime.state.as_str(),
                    "started" | "stale" | "degraded" | "warming"
                ) {
                    if let Some(listen_addr) = runtime.listen_addr.as_deref() {
                        if let Ok(response) = request_check_path(listen_addr, &changed) {
                            return render_raw_value(
                                response.value,
                                format,
                                exit_code_from_i32(response.exit_code),
                            );
                        }
                    }
                }
            }
            match core.check_changed_path(changed) {
                Ok(report) => render_success(
                    DaemonCheckPathOutput {
                        schema: "assura.daemon.check_path.v1",
                        health: core.health(),
                        report,
                    },
                    format,
                ),
                Err(error) => render_error(error, format, Some(core.health())),
            }
        }
        DaemonCommands::References { limit, .. } => {
            match reference_request.expect("references request prevalidated") {
                DaemonReferenceRequest::Source(path) => {
                    render_reference(core.changed_source_references(path, limit), format, &core)
                }
                DaemonReferenceRequest::Target(path) => {
                    render_reference(core.changed_target_references(path, limit), format, &core)
                }
                DaemonReferenceRequest::MovedTarget {
                    previous_path,
                    new_path,
                } => render_moved_reference(
                    core.moved_target_references(previous_path, new_path, limit),
                    format,
                    &core,
                ),
            }
        }
        DaemonCommands::Status { .. }
        | DaemonCommands::Start { .. }
        | DaemonCommands::Stop { .. }
        | DaemonCommands::Restart { .. }
        | DaemonCommands::Doctor { .. }
        | DaemonCommands::Logs { .. }
        | DaemonCommands::Serve { .. } => {
            unreachable!("management preview commands return before daemon core load")
        }
    }
}

impl DaemonCommands {
    fn format(&self) -> OutputFormat {
        match self {
            Self::Status { format, .. }
            | Self::Start { format, .. }
            | Self::Stop { format, .. }
            | Self::Restart { format, .. }
            | Self::Doctor { format, .. }
            | Self::Logs { format, .. }
            | Self::Health { format, .. }
            | Self::CheckPath { format, .. }
            | Self::References { format, .. } => *format,
            Self::Serve { .. } => OutputFormat::Text,
        }
    }

    fn path(&self) -> PathBuf {
        let path = match self {
            Self::Status { path, .. }
            | Self::Start { path, .. }
            | Self::Stop { path, .. }
            | Self::Restart { path, .. }
            | Self::Doctor { path, .. }
            | Self::Logs { path, .. }
            | Self::Health { path, .. }
            | Self::CheckPath { path, .. }
            | Self::References { path, .. } => path.clone(),
            Self::Serve { path, .. } => Some(path.clone()),
        };
        path.unwrap_or_else(|| PathBuf::from("."))
    }
}

fn render_reference(
    result: Result<DaemonAffectedReferences, DaemonCoreError>,
    format: OutputFormat,
    core: &LocalDaemonCore,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
    match result {
        Ok(response) => render_success(response, format),
        Err(error) => render_error(error, format, Some(core.health())),
    }
}

fn render_moved_reference(
    result: Result<DaemonMovedTargetReferences, DaemonCoreError>,
    format: OutputFormat,
    core: &LocalDaemonCore,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
    match result {
        Ok(response) => render_success(response, format),
        Err(error) => render_error(error, format, Some(core.health())),
    }
}

#[derive(Debug, Serialize)]
struct DaemonCheckPathOutput {
    schema: &'static str,
    health: DaemonHealth,
    report: StructureCheckReport,
}
