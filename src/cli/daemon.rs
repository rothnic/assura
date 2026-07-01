//! CLI probes for daemon-ready local project state.

#[path = "daemon_management.rs"]
mod management;
#[path = "daemon_text.rs"]
mod text;

use super::{ExitCode, OutputFormat};
use crate::cli::check::StructureCheckReport;
use crate::daemon::{
    DaemonAffectedReferences, DaemonCoreError, DaemonHealth, DaemonMovedTargetReferences,
    LocalDaemonCore,
};
use clap::Subcommand;
use management::{daemon_doctor_output, daemon_status_output, health_for_path};
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

    /// Report daemon management diagnostics and remediation commands.
    Doctor {
        /// Project root directory (defaults to current directory).
        path: Option<PathBuf>,

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
}

/// Run a daemon-ready probe command.
pub async fn daemon_command(command: DaemonCommands, config: Option<PathBuf>) -> ExitCode {
    match run_daemon_command(command, config) {
        Ok(outcome) => {
            println!("{}", outcome.rendered);
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
        _ => {}
    }
    let mut core = match LocalDaemonCore::load(path, config) {
        Ok(core) => core,
        Err(error) => return render_load_error(error, format, &command),
    };

    match command {
        DaemonCommands::Health { .. } => render_success(core.health(), format),
        DaemonCommands::CheckPath { changed, .. } => match core.check_changed_path(changed) {
            Ok(report) => render_success(
                DaemonCheckPathOutput {
                    schema: "assura.daemon.check_path.v1",
                    health: core.health(),
                    report,
                },
                format,
            ),
            Err(error) => render_error(error, format, Some(core.health())),
        },
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
        DaemonCommands::Status { .. } | DaemonCommands::Doctor { .. } => {
            unreachable!("management preview commands return before daemon core load")
        }
    }
}

impl DaemonCommands {
    fn format(&self) -> OutputFormat {
        match self {
            Self::Status { format, .. }
            | Self::Doctor { format, .. }
            | Self::Health { format, .. }
            | Self::CheckPath { format, .. }
            | Self::References { format, .. } => *format,
        }
    }

    fn path(&self) -> PathBuf {
        let path = match self {
            Self::Status { path, .. }
            | Self::Doctor { path, .. }
            | Self::Health { path, .. }
            | Self::CheckPath { path, .. }
            | Self::References { path, .. } => path.clone(),
        };
        path.unwrap_or_else(|| PathBuf::from("."))
    }
}

enum DaemonReferenceRequest {
    Source(PathBuf),
    Target(PathBuf),
    MovedTarget {
        previous_path: PathBuf,
        new_path: PathBuf,
    },
}

fn reference_request(
    source: Option<PathBuf>,
    target: Option<PathBuf>,
    moved_target: Option<PathBuf>,
    new_target: Option<PathBuf>,
) -> Result<DaemonReferenceRequest, String> {
    let selected = source.is_some() as u8 + target.is_some() as u8 + moved_target.is_some() as u8;
    if selected != 1 {
        return Err(
            "daemon references requires exactly one of --source, --target, or --moved-target"
                .to_string(),
        );
    }
    if let Some(path) = source {
        return Ok(DaemonReferenceRequest::Source(path));
    }
    if let Some(path) = target {
        return Ok(DaemonReferenceRequest::Target(path));
    }
    let previous_path = moved_target.expect("moved_target selected");
    let Some(new_path) = new_target else {
        return Err("--moved-target requires --new-target".to_string());
    };
    Ok(DaemonReferenceRequest::MovedTarget {
        previous_path,
        new_path,
    })
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

fn render_success<T>(
    value: T,
    format: OutputFormat,
) -> Result<DaemonCommandOutcome, DaemonCommandError>
where
    T: Serialize + DaemonTextRender,
{
    render_success_with_exit(value, format, ExitCode::Success)
}

fn render_success_with_exit<T>(
    value: T,
    format: OutputFormat,
    exit_code: ExitCode,
) -> Result<DaemonCommandOutcome, DaemonCommandError>
where
    T: Serialize + DaemonTextRender,
{
    Ok(DaemonCommandOutcome {
        rendered: render(value, format)?,
        exit_code,
    })
}

fn render_error(
    error: DaemonCoreError,
    format: OutputFormat,
    fallback_health: Option<DaemonHealth>,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
    match error {
        DaemonCoreError::Stale(health) => Ok(DaemonCommandOutcome {
            rendered: render(
                DaemonErrorOutput {
                    schema: "assura.daemon.error.v1",
                    error: "daemon state is stale",
                    health,
                },
                format,
            )?,
            exit_code: ExitCode::RuntimeError,
        }),
        other if matches!(format, OutputFormat::Json | OutputFormat::Yaml) => {
            let message = other.to_string();
            let health = fallback_health
                .map(|health| {
                    DaemonHealth::unavailable(
                        health.project_root,
                        health.config_path,
                        message.clone(),
                    )
                })
                .unwrap_or_else(|| {
                    DaemonHealth::unavailable(
                        PathBuf::from("."),
                        PathBuf::from(".assura/config.yml"),
                        message.clone(),
                    )
                });
            Ok(DaemonCommandOutcome {
                rendered: render(
                    DaemonErrorOutput {
                        schema: "assura.daemon.error.v1",
                        error: "daemon state is unavailable",
                        health,
                    },
                    format,
                )?,
                exit_code: ExitCode::RuntimeError,
            })
        }
        other => Err(DaemonCommandError {
            message: other.to_string(),
            exit_code: ExitCode::RuntimeError,
        }),
    }
}

fn render_load_error(
    error: DaemonCoreError,
    format: OutputFormat,
    command: &DaemonCommands,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
    let project_root = command.path();
    let config_path = project_root.join(".assura/config.yml");
    let reason = error.to_string();
    let health = DaemonHealth::unavailable(project_root, config_path, reason);
    render_error(error, format, Some(health))
}

fn render<T>(value: T, format: OutputFormat) -> Result<String, DaemonCommandError>
where
    T: Serialize + DaemonTextRender,
{
    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&value).map_err(|error| DaemonCommandError {
                message: error.to_string(),
                exit_code: ExitCode::RuntimeError,
            })
        }
        OutputFormat::Yaml => serde_yaml::to_string(&value).map_err(|error| DaemonCommandError {
            message: error.to_string(),
            exit_code: ExitCode::RuntimeError,
        }),
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => Ok(value.render_text()),
    }
}

struct DaemonCommandOutcome {
    rendered: String,
    exit_code: ExitCode,
}

struct DaemonCommandError {
    message: String,
    exit_code: ExitCode,
}

#[derive(Debug, Serialize)]
struct DaemonCheckPathOutput {
    schema: &'static str,
    health: DaemonHealth,
    report: StructureCheckReport,
}

#[derive(Debug, Serialize)]
struct DaemonErrorOutput {
    schema: &'static str,
    error: &'static str,
    health: DaemonHealth,
}

trait DaemonTextRender {
    fn render_text(&self) -> String;
}
