//! File line budgets for an explicit Git tree comparison.

use super::file_limits::{pattern_severity, repair_message};
use super::report::FindingSeverity;
use super::StructureChecker;
use crate::cli::args::{ChangeCheckFormat, ExitCode};
use crate::cli::git_snapshot::{Change, Scope, Side, Snapshot, Tree};
use crate::config::loader::ConfigLoader;
use serde::Serialize;
use std::path::Path;

const POLICY: &str = ".assura/config.yml";

#[derive(Serialize)]
struct Report {
    success: bool,
    scope: &'static str,
    baseline: TreeIdentity,
    selected: TreeIdentity,
    policy_path: &'static str,
    files_checked: usize,
    findings: Vec<Finding>,
}

#[derive(Serialize)]
struct TreeIdentity {
    tree: String,
    commit: Option<String>,
}

impl From<Tree> for TreeIdentity {
    fn from(tree: Tree) -> Self {
        Self {
            tree: tree.oid,
            commit: tree.commit,
        }
    }
}

#[derive(Serialize)]
struct Finding {
    path: String,
    rule: &'static str,
    severity: &'static str,
    blocking: bool,
    actual: usize,
    limit: usize,
    baseline_lines: Option<usize>,
    allowed_lines: usize,
    repair_message: Option<String>,
}

pub(in crate::cli) fn command(scope: Scope<'_>, format: ChangeCheckFormat) -> ExitCode {
    match run(scope).and_then(|report| {
        match format {
            ChangeCheckFormat::Json => {
                let output = serde_json::to_string_pretty(&report)
                    .map_err(|error| (ExitCode::RuntimeError, error.to_string()))?;
                println!("{output}");
            }
            ChangeCheckFormat::Text => report.print_text(),
        }
        Ok(report.success)
    }) {
        Ok(true) => ExitCode::Success,
        Ok(false) => ExitCode::ValidationFailed,
        Err((code, error)) => {
            eprintln!("Error: {error}");
            code
        }
    }
}

impl Report {
    fn print_text(&self) {
        println!(
            "{} max_lines check: {} → {} (policy {}, {} files checked)",
            self.scope,
            self.baseline.tree,
            self.selected.tree,
            self.policy_path,
            self.files_checked,
        );
        for finding in &self.findings {
            println!(
                "[{}] {:?}: {} lines exceeds the allowed {} (configured limit {}). Lines to cut or move: {}.",
                finding.severity, finding.path, finding.actual, finding.allowed_lines, finding.limit,
                finding.actual - finding.allowed_lines,
            );
            if let Some(message) = &finding.repair_message {
                println!("  Policy message: {}", message.escape_debug());
            }
        }
    }
}

fn run(scope: Scope<'_>) -> Result<Report, (ExitCode, String)> {
    let runtime_error = |error| (ExitCode::RuntimeError, error);
    let config_error = |error| (ExitCode::ConfigurationError, error);
    let cwd = std::env::current_dir().map_err(|error| runtime_error(error.to_string()))?;
    let snapshot = Snapshot::resolve(&cwd, scope).map_err(runtime_error)?;
    let policy = snapshot
        .read_file(Side::Selected, Path::new(POLICY))
        .map_err(config_error)?
        .ok_or_else(|| {
            (
                ExitCode::NoConfigFound,
                format!(
                    "{POLICY} is absent from selected tree {}",
                    snapshot.selected.oid
                ),
            )
        })?;
    let policy =
        std::str::from_utf8(&policy).map_err(|error| config_error(format!("{POLICY}: {error}")))?;
    let config = ConfigLoader::parse(policy)
        .map_err(|error| config_error(format!("{POLICY} in {}: {error}", snapshot.selected.oid)))?;
    let mut checker = StructureChecker::new(snapshot.root.clone(), config, false);
    let mut findings = Vec::new();
    let mut files_checked = 0;
    for change in &snapshot.changes {
        let (path, previous) = match change {
            Change::Added(path) | Change::TypeChanged(path) => (path, None),
            Change::Modified(path) => (path, Some(path)),
            Change::Renamed { from, to } => (to, Some(from)),
            Change::Deleted(_) => continue,
        };
        let display_path = git_path_string(path).map_err(runtime_error)?;
        let Some((limit, files, pattern)) = selected_budget(&mut checker, path) else {
            continue;
        };
        let Some(actual) = count_lines(&snapshot, Side::Selected, path).map_err(runtime_error)?
        else {
            continue;
        };
        files_checked += 1;
        if actual <= limit {
            continue;
        }
        let baseline_lines = match previous {
            Some(previous)
                if previous == path
                    || selected_budget(&mut checker, previous)
                        .is_some_and(|(previous_limit, _, _)| previous_limit == limit) =>
            {
                count_lines(&snapshot, Side::Baseline, previous).map_err(runtime_error)?
            }
            _ => None,
        };
        let allowed_lines = limit.max(baseline_lines.unwrap_or(0));
        if actual > allowed_lines {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let severity = pattern_severity(
                &files,
                pattern.as_deref(),
                filename,
                path,
                &checker.glob_patterns,
            );
            let severity = FindingSeverity::parse_or_default(&severity);
            let repair_message = repair_message(
                &files,
                pattern.as_deref(),
                filename,
                path,
                &checker.glob_patterns,
            );
            findings.push(Finding {
                path: display_path,
                rule: "max_lines",
                severity: severity.as_str(),
                blocking: severity.is_blocking(),
                actual,
                limit,
                baseline_lines,
                allowed_lines,
                repair_message: repair_message.map(str::to_owned),
            });
        }
    }
    findings.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Report {
        success: !findings.iter().any(|finding| finding.blocking),
        scope: match scope {
            Scope::Staged => "staged",
            Scope::Direct { .. } => "direct",
            Scope::MergeBase { .. } => "merge-base",
        },
        baseline: snapshot.baseline.into(),
        selected: snapshot.selected.into(),
        policy_path: POLICY,
        files_checked,
        findings,
    })
}

fn selected_budget(
    checker: &mut StructureChecker,
    path: &Path,
) -> Option<(
    usize,
    std::sync::Arc<crate::config::config::FileBundle>,
    Option<String>,
)> {
    if path.ancestors().any(|part| checker.is_excluded_rel(part)) {
        return None;
    }
    let filename = path.file_name()?.to_str()?;
    let rules = checker.resolve_rules(path.parent()?);
    let files = rules.files?;
    let (limit, pattern) = checker.file_line_budget(path, &files, filename)?;
    let pattern = pattern.map(str::to_owned);
    Some((limit, files, pattern))
}

fn count_lines(snapshot: &Snapshot, side: Side, path: &Path) -> Result<Option<usize>, String> {
    let Some(bytes) = snapshot.read_regular_file(side, path)? else {
        return Ok(None);
    };
    let content = std::str::from_utf8(&bytes)
        .map_err(|error| format!("cannot count lines in Git blob {path:?}: {error}"))?;
    Ok(Some(content.lines().count()))
}

fn git_path_string(path: &Path) -> Result<String, String> {
    let path_string = path
        .to_str()
        .ok_or_else(|| format!("unsupported non-UTF-8 Git path {path:?}"))?;
    #[cfg(windows)]
    return Ok(path_string.replace('\\', "/"));
    #[cfg(not(windows))]
    Ok(path_string.to_owned())
}
