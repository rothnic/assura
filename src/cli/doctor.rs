//! Project doctor and path explanation commands.

use super::args::CheckOutputFormat;
use super::check::{
    explain_structure_path, run_structure_check_with_target_mode, CheckError, CheckTargetMode,
    PathExplainReport,
};
pub(super) use super::doctor_report::{
    DoctorItem, DoctorNextAction, DoctorViolation, ProjectDoctorBuild, ProjectDoctorReport,
};
use super::ExitCode;
use crate::config::loader::ConfigLoader;
use std::path::{Path, PathBuf};

pub(super) const DOCTOR_AGENT_SCHEMA: &str = "assura.project-doctor.agent.v1";

/// Run the top-level project doctor command.
pub async fn doctor_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    format: CheckOutputFormat,
) -> ExitCode {
    match build_project_doctor(path, config) {
        Ok(report) => {
            println!("{}", render_doctor(&report, format));
            if report.check.status == "fail" {
                ExitCode::ValidationFailed
            } else {
                ExitCode::Success
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            exit_code_for_check_error(&error)
        }
    }
}

/// Run the top-level path explanation command.
pub async fn explain_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    format: CheckOutputFormat,
) -> ExitCode {
    match explain_structure_path(path, config) {
        Ok(report) => {
            println!("{}", render_explain(&report, format));
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {error}");
            exit_code_for_check_error(&error)
        }
    }
}

pub(super) fn project_doctor_packet_json(
    project_root: &Path,
    config: Option<PathBuf>,
) -> Result<String, String> {
    let report = build_project_doctor(Some(project_root.to_path_buf()), config)
        .map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
}

pub(super) fn build_project_doctor(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
) -> Result<ProjectDoctorReport, CheckError> {
    build_project_doctor_with_structure_report(path, config).map(|build| build.doctor)
}

pub(super) fn build_project_doctor_with_structure_report(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
) -> Result<ProjectDoctorBuild, CheckError> {
    let report =
        run_structure_check_with_target_mode(path, config, false, CheckTargetMode::Recursive)?;
    let config = ConfigLoader::load(&report.config_path)?;
    let doctor = ProjectDoctorReport::from_check(&report, &config);
    Ok(ProjectDoctorBuild {
        doctor,
        structure_report: report,
    })
}

fn render_doctor(report: &ProjectDoctorReport, format: CheckOutputFormat) -> String {
    match format {
        CheckOutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        CheckOutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        CheckOutputFormat::Agent => super::doctor_agent::render_agent_doctor(report),
        CheckOutputFormat::Text | CheckOutputFormat::Advice | CheckOutputFormat::Status => {
            report.render_text()
        }
    }
}

fn render_explain(report: &PathExplainReport, format: CheckOutputFormat) -> String {
    match format {
        CheckOutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        CheckOutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        CheckOutputFormat::Agent => super::doctor_agent::render_agent_explain(report),
        CheckOutputFormat::Text | CheckOutputFormat::Advice | CheckOutputFormat::Status => {
            report.render_text()
        }
    }
}

pub(super) fn exit_code_for_check_error(error: &CheckError) -> ExitCode {
    match error {
        CheckError::NoConfig(_) => ExitCode::NoConfigFound,
        CheckError::MissingPath(_)
        | CheckError::OutsideProject { .. }
        | CheckError::InvalidConfigLocation(_) => ExitCode::ConfigurationError,
        CheckError::Config(_) => ExitCode::ConfigurationError,
        CheckError::Io(_) | CheckError::WalkDir(_) | CheckError::Walkdir(_) => {
            ExitCode::RuntimeError
        }
    }
}
