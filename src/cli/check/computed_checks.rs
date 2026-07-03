//! Project-local script-backed computed check execution.

use super::{StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::ComputedCheckConfig;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const COMPUTED_CHECK_INPUT_SCHEMA: &str = "assura.computed-check.input.v1";
const COMPUTED_CHECK_OUTPUT_SCHEMA: &str = "assura.computed-check.output.v1";
const STDERR_LIMIT: usize = 300;

#[derive(Debug, Deserialize)]
struct ComputedCheckOutput {
    schema: String,
    #[serde(default)]
    findings: Vec<ComputedCheckFinding>,
}

#[derive(Debug, Deserialize)]
struct ComputedCheckFinding {
    code: String,
    message: String,
    path: Option<String>,
    severity: Option<String>,
    metadata: Option<serde_json::Value>,
}

impl StructureChecker {
    pub(super) fn validate_computed_checks(
        &self,
        policies: &[ComputedCheckConfig],
        report: &mut StructureCheckReport,
    ) {
        for policy in policies {
            self.validate_computed_check(policy, report);
        }
    }

    fn validate_computed_check(
        &self,
        policy: &ComputedCheckConfig,
        report: &mut StructureCheckReport,
    ) {
        let selected_script = platform_script(policy);
        let script_abs = self.project_root.join(Path::new(selected_script));
        if !script_abs.is_file() {
            self.push_computed_check_violation(
                report,
                policy,
                self.relative_path(&report.config_path),
                "script_missing",
                format!(
                    "Computed check `{}` script `{}` does not exist",
                    policy.id, selected_script
                ),
                None,
            );
            return;
        }

        let script_abs = match project_local_script(&self.project_root, &script_abs) {
            Ok(script_abs) => script_abs,
            Err(error) => {
                self.push_computed_check_violation(
                    report,
                    policy,
                    self.relative_path(&report.config_path),
                    error.code,
                    error.message,
                    None,
                );
                return;
            }
        };

        match run_computed_check_script(policy, &script_abs, report) {
            Ok(output) => self.ingest_computed_check_output(policy, output, report),
            Err(error) => {
                self.push_computed_check_violation(
                    report,
                    policy,
                    self.relative_path(&report.config_path),
                    error.code,
                    error.message,
                    None,
                );
            }
        }
    }

    fn ingest_computed_check_output(
        &self,
        policy: &ComputedCheckConfig,
        output: ComputedCheckOutput,
        report: &mut StructureCheckReport,
    ) {
        if output.schema != COMPUTED_CHECK_OUTPUT_SCHEMA {
            self.push_computed_check_violation(
                report,
                policy,
                self.relative_path(&report.config_path),
                "invalid_output",
                format!(
                    "Computed check `{}` output schema must be `{}`",
                    policy.id, COMPUTED_CHECK_OUTPUT_SCHEMA
                ),
                None,
            );
            return;
        }

        for finding in output.findings {
            if !valid_identifier(&finding.code) || finding.message.trim().is_empty() {
                self.push_computed_check_violation(
                    report,
                    policy,
                    self.relative_path(&report.config_path),
                    "invalid_output",
                    format!(
                        "Computed check `{}` emitted a finding with invalid code or message",
                        policy.id
                    ),
                    None,
                );
                continue;
            }
            let severity = match finding.severity.as_deref() {
                Some(severity) if !valid_severity(severity) => {
                    self.push_computed_check_violation(
                        report,
                        policy,
                        self.relative_path(&report.config_path),
                        "invalid_output",
                        format!(
                            "Computed check `{}` emitted invalid severity `{severity}`",
                            policy.id
                        ),
                        None,
                    );
                    continue;
                }
                severity => severity,
            };
            let path = match finding.path.as_deref() {
                Some(path) => match safe_project_relative_path(path) {
                    Some(path) => path,
                    None => {
                        self.push_computed_check_violation(
                            report,
                            policy,
                            self.relative_path(&report.config_path),
                            "invalid_output",
                            format!(
                                "Computed check `{}` emitted an unsafe finding path `{path}`",
                                policy.id
                            ),
                            None,
                        );
                        continue;
                    }
                },
                None => self.relative_path(&report.config_path),
            };
            self.push_computed_check_violation_with_metadata(
                report,
                policy,
                ComputedCheckViolationInput {
                    path,
                    code: finding.code,
                    message: finding.message,
                    severity: severity.map(str::to_string),
                    metadata: finding.metadata,
                },
            );
        }
    }

    fn push_computed_check_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &ComputedCheckConfig,
        path: PathBuf,
        code: impl AsRef<str>,
        message: String,
        severity: Option<&str>,
    ) {
        self.push_computed_check_violation_with_metadata(
            report,
            policy,
            ComputedCheckViolationInput {
                path,
                code: code.as_ref().to_string(),
                message,
                severity: severity.map(str::to_string),
                metadata: None,
            },
        );
    }

    fn push_computed_check_violation_with_metadata(
        &self,
        report: &mut StructureCheckReport,
        policy: &ComputedCheckConfig,
        violation: ComputedCheckViolationInput,
    ) {
        report.violations.push(
            StructureViolation::new(
                violation.path,
                format!("computed_check:{}:{}", policy.id, violation.code),
                violation.message,
                violation
                    .severity
                    .as_deref()
                    .or(policy.severity.as_deref())
                    .unwrap_or("medium"),
            )
            .with_metadata(violation.metadata),
        );
    }
}

struct ComputedCheckViolationInput {
    path: PathBuf,
    code: String,
    message: String,
    severity: Option<String>,
    metadata: Option<serde_json::Value>,
}

struct ScriptExecutionError {
    code: &'static str,
    message: String,
}

fn run_computed_check_script(
    policy: &ComputedCheckConfig,
    script_abs: &Path,
    report: &StructureCheckReport,
) -> Result<ComputedCheckOutput, ScriptExecutionError> {
    let mut command = command_for_script(script_abs);
    let mut child = command
        .args(&policy.args)
        .current_dir(&report.project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| ScriptExecutionError {
            code: "spawn_failed",
            message: format!(
                "Computed check `{}` failed to start `{}`: {error}",
                policy.id, policy.script
            ),
        })?;

    let request = json!({
        "schema": COMPUTED_CHECK_INPUT_SCHEMA,
        "id": policy.id,
        "checked_path": report.checked_path,
        "config_path": report.config_path,
    });
    if let Some(mut stdin) = child.stdin.take() {
        let payload = serde_json::to_vec(&request).map_err(|error| ScriptExecutionError {
            code: "input_failed",
            message: format!("Computed check `{}` input failed: {error}", policy.id),
        })?;
        stdin
            .write_all(&payload)
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdin.flush())
            .map_err(|error| ScriptExecutionError {
                code: "input_failed",
                message: format!(
                    "Computed check `{}` failed to write stdin: {error}",
                    policy.id
                ),
            })?;
        drop(stdin);
    }

    let timeout = Duration::from_millis(policy.timeout_ms);
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let output = child
                    .wait_with_output()
                    .map_err(|error| ScriptExecutionError {
                        code: "timeout",
                        message: format!(
                            "Computed check `{}` timed out after {} ms and cleanup failed: {error}",
                            policy.id, policy.timeout_ms
                        ),
                    })?;
                let stderr = truncate_stderr(&output.stderr);
                return Err(ScriptExecutionError {
                    code: "timeout",
                    message: format!(
                        "Computed check `{}` timed out after {} ms{}",
                        policy.id, policy.timeout_ms, stderr
                    ),
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                return Err(ScriptExecutionError {
                    code: "runtime_failed",
                    message: format!("Computed check `{}` wait failed: {error}", policy.id),
                });
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| ScriptExecutionError {
            code: "runtime_failed",
            message: format!(
                "Computed check `{}` output collection failed: {error}",
                policy.id
            ),
        })?;
    if !output.status.success() {
        return Err(ScriptExecutionError {
            code: "nonzero_exit",
            message: format!(
                "Computed check `{}` exited with status {}{}",
                policy.id,
                output.status,
                truncate_stderr(&output.stderr)
            ),
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|error| ScriptExecutionError {
        code: "invalid_output",
        message: format!(
            "Computed check `{}` emitted invalid JSON output: {error}",
            policy.id
        ),
    })
}

fn platform_script(policy: &ComputedCheckConfig) -> &str {
    #[cfg(windows)]
    if let Some(script) = policy.windows_script.as_deref() {
        return script;
    }
    policy.script.as_str()
}

fn command_for_script(script_abs: &Path) -> Command {
    #[cfg(windows)]
    {
        if script_abs
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(extension.to_ascii_lowercase().as_str(), "bat" | "cmd")
            })
        {
            let mut command = Command::new("cmd.exe");
            command
                .arg("/D")
                .arg("/C")
                .arg("call")
                .arg(cmd_compatible_script_path(script_abs));
            return command;
        }
    }
    Command::new(script_abs)
}

#[cfg_attr(not(windows), allow(dead_code))]
fn cmd_compatible_script_path(script_abs: &Path) -> PathBuf {
    let path = script_abs.to_string_lossy();
    let Some(stripped) = path.strip_prefix(r"\\?\") else {
        return script_abs.to_path_buf();
    };
    if let Some(unc) = stripped.strip_prefix(r"UNC\") {
        return PathBuf::from(format!(r"\\{unc}"));
    }
    PathBuf::from(stripped)
}

fn truncate_stderr(stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if stderr.is_empty() {
        return String::new();
    }
    let mut truncated = stderr.chars().take(STDERR_LIMIT).collect::<String>();
    if stderr.chars().count() > STDERR_LIMIT {
        truncated.push_str("...");
    }
    format!("; stderr: {truncated}")
}

fn safe_project_relative_path(value: &str) -> Option<PathBuf> {
    if value.trim().is_empty() {
        return None;
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn project_local_script(
    project_root: &Path,
    script_abs: &Path,
) -> Result<PathBuf, ScriptExecutionError> {
    let root = project_root
        .canonicalize()
        .map_err(|error| ScriptExecutionError {
            code: "runtime_failed",
            message: format!(
                "Computed check project root `{}` could not be resolved: {error}",
                project_root.display()
            ),
        })?;
    let script = script_abs
        .canonicalize()
        .map_err(|error| ScriptExecutionError {
            code: "spawn_failed",
            message: format!(
                "Computed check script `{}` could not be resolved: {error}",
                script_abs.display()
            ),
        })?;
    if !script.starts_with(&root) {
        return Err(ScriptExecutionError {
            code: "script_outside_project",
            message: format!(
                "Computed check script `{}` resolves outside the project root",
                script_abs.display()
            ),
        });
    }
    Ok(script)
}

fn valid_identifier(value: &str) -> bool {
    !value.trim().is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
}

fn valid_severity(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "low" | "medium" | "high" | "critical"
    )
}

#[cfg(test)]
mod tests {
    use super::cmd_compatible_script_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn cmd_compatible_script_path_strips_extended_drive_prefix() {
        assert_eq!(
            cmd_compatible_script_path(Path::new(r"\\?\C:\Temp\rollup.cmd")),
            PathBuf::from(r"C:\Temp\rollup.cmd")
        );
    }

    #[test]
    fn cmd_compatible_script_path_strips_extended_unc_prefix() {
        assert_eq!(
            cmd_compatible_script_path(Path::new(r"\\?\UNC\server\share\rollup.cmd")),
            PathBuf::from(r"\\server\share\rollup.cmd")
        );
    }
}
