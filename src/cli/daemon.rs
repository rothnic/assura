//! CLI probes for daemon-ready local project state.

use super::{ExitCode, OutputFormat};
use crate::cli::check::StructureCheckReport;
use crate::daemon::{
    DaemonAffectedReferences, DaemonCoreError, DaemonHealth, DaemonMovedTargetReferences,
    LocalDaemonCore,
};
use clap::Subcommand;
use serde::Serialize;
use std::path::PathBuf;

/// Daemon-ready probe commands.
#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
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
    }
}

impl DaemonCommands {
    fn format(&self) -> OutputFormat {
        match self {
            Self::Health { format, .. }
            | Self::CheckPath { format, .. }
            | Self::References { format, .. } => *format,
        }
    }

    fn path(&self) -> PathBuf {
        let path = match self {
            Self::Health { path, .. }
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
    Ok(DaemonCommandOutcome {
        rendered: render(value, format)?,
        exit_code: ExitCode::Success,
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

impl DaemonTextRender for DaemonHealth {
    fn render_text(&self) -> String {
        format!(
            "Daemon health: {:?}\nreason={}\ngeneration={}\nproject_root={}\nconfig_path={}\nstatus_file={}\nlog_file={}\nfallback={}",
            self.state,
            self.reason,
            self.generation,
            self.project_root.display(),
            self.config_path.display(),
            self.runtime_paths.status_file.display(),
            self.runtime_paths.log_file.display(),
            self.fallback_command,
        )
    }
}

impl DaemonTextRender for DaemonCheckPathOutput {
    fn render_text(&self) -> String {
        format!(
            "{}\nchanged_path_success={}\nviolations={}",
            self.health.render_text(),
            self.report.success,
            self.report.violations.len()
        )
    }
}

impl DaemonTextRender for DaemonAffectedReferences {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Daemon references: {} {} ({}/{}, truncated={})",
            self.mode,
            self.path.display(),
            self.bounds.returned,
            self.bounds.limit,
            self.bounds.truncated
        )];
        lines.push(format!(
            "health={:?} reason={}",
            self.health.state, self.health.reason
        ));
        for reference in &self.references {
            lines.push(format!(
                "source={}:{}:{} target={} anchor={} lines={} exists={} rule={} kind={} confidence={}",
                reference.source_path.display(),
                optional_usize(reference.source_line),
                optional_usize(reference.source_column),
                reference.target_path.display(),
                optional_string(reference.target_anchor.as_deref()),
                target_lines(reference.target_line_start, reference.target_line_end),
                reference.target_exists,
                reference.rule,
                reference.reference_kind,
                reference.confidence,
            ));
        }
        lines.join("\n")
    }
}

impl DaemonTextRender for DaemonMovedTargetReferences {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Daemon moved-target references: {} -> {} ({}/{}, truncated={})",
            self.previous_path.display(),
            self.new_path.display(),
            self.bounds.returned,
            self.bounds.limit,
            self.bounds.truncated
        )];
        lines.push(format!(
            "health={:?} reason={}",
            self.health.state, self.health.reason
        ));
        for reference in &self.references {
            lines.push(format!(
                "source={}:{}:{} target={} anchor={} lines={} exists={} rule={} kind={} confidence={}",
                reference.source_path.display(),
                optional_usize(reference.source_line),
                optional_usize(reference.source_column),
                reference.target_path.display(),
                optional_string(reference.target_anchor.as_deref()),
                target_lines(reference.target_line_start, reference.target_line_end),
                reference.target_exists,
                reference.rule,
                reference.reference_kind,
                reference.confidence,
            ));
        }
        lines.join("\n")
    }
}

impl DaemonTextRender for DaemonErrorOutput {
    fn render_text(&self) -> String {
        format!("{}\nerror={}", self.health.render_text(), self.error)
    }
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn optional_string(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

fn target_lines(start: Option<usize>, end: Option<usize>) -> String {
    match (start, end) {
        (Some(start), Some(end)) if start != end => format!("{start}-{end}"),
        (Some(start), _) => start.to_string(),
        _ => "-".to_string(),
    }
}
