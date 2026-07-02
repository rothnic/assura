//! Rendering helpers for daemon CLI outputs.

use super::OutputFormat;
use crate::cli::ExitCode;
use crate::daemon::{DaemonCoreError, DaemonHealth};
use serde::Serialize;
use std::path::PathBuf;

pub(super) fn render_success<T>(
    value: T,
    format: OutputFormat,
) -> Result<DaemonCommandOutcome, DaemonCommandError>
where
    T: Serialize + DaemonTextRender,
{
    render_success_with_exit(value, format, ExitCode::Success)
}

pub(super) fn render_success_with_exit<T>(
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

pub(super) fn render_raw_value(
    value: serde_json::Value,
    format: OutputFormat,
    exit_code: ExitCode,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
    let rendered = match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&value).map_err(|error| DaemonCommandError {
                message: error.to_string(),
                exit_code: ExitCode::RuntimeError,
            })?
        }
        OutputFormat::Yaml => {
            serde_yaml::to_string(&value).map_err(|error| DaemonCommandError {
                message: error.to_string(),
                exit_code: ExitCode::RuntimeError,
            })?
        }
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => value.to_string(),
    };
    Ok(DaemonCommandOutcome {
        rendered,
        exit_code,
    })
}

pub(super) fn render_error(
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
                    health: *health,
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

pub(super) fn render_load_error(
    error: DaemonCoreError,
    format: OutputFormat,
    project_root: PathBuf,
) -> Result<DaemonCommandOutcome, DaemonCommandError> {
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

pub(super) struct DaemonCommandOutcome {
    pub(super) rendered: String,
    pub(super) exit_code: ExitCode,
}

pub(super) struct DaemonCommandError {
    pub(super) message: String,
    pub(super) exit_code: ExitCode,
}

#[derive(Debug, Serialize)]
struct DaemonErrorOutput {
    schema: &'static str,
    error: &'static str,
    health: DaemonHealth,
}

impl DaemonTextRender for DaemonErrorOutput {
    fn render_text(&self) -> String {
        format!("{}\nerror={}", self.health.render_text(), self.error)
    }
}

pub(crate) trait DaemonTextRender {
    fn render_text(&self) -> String;
}

pub(super) fn exit_code_from_i32(value: i32) -> ExitCode {
    match value {
        0 => ExitCode::Success,
        1 => ExitCode::ValidationFailed,
        2 => ExitCode::ConfigurationError,
        4 => ExitCode::NoConfigFound,
        _ => ExitCode::RuntimeError,
    }
}
