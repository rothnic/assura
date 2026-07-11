//! Stable finding fingerprints and worktree-scoped review history.

use super::report::ProjectReviewFinding;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HISTORY_SCHEMA: &str = "assura.project-review-history.v3";

#[derive(Debug, Clone, Serialize)]
pub(super) struct FindingHistorySummary {
    pub(super) new: usize,
    pub(super) worsened: usize,
    pub(super) unchanged: usize,
    pub(super) resolved: usize,
    pub(super) cache: FindingHistoryCache,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct FindingHistoryCache {
    pub(super) mode: &'static str,
    pub(super) path: String,
    pub(super) loaded: bool,
    pub(super) written: bool,
    pub(super) fallback_reason: Option<&'static str>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FindingHistoryFile {
    schema: String,
    findings: Vec<FindingSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FindingSnapshot {
    fingerprint: String,
    id: String,
    category: String,
    severity: String,
    action_kind: String,
    title: String,
    detail: String,
    detail_hash: String,
    #[serde(default)]
    pressure: Option<usize>,
    command: String,
    source: String,
}

pub(super) fn apply_finding_history(
    project_root: &str,
    findings: &mut Vec<ProjectReviewFinding>,
    persist: bool,
) -> FindingHistorySummary {
    for finding in findings.iter_mut() {
        finding.fingerprint = finding_fingerprint(finding);
    }

    if !persist {
        return FindingHistorySummary {
            new: findings.len(),
            worsened: 0,
            unchanged: 0,
            resolved: 0,
            cache: FindingHistoryCache {
                mode: "disabled",
                path: String::new(),
                loaded: false,
                written: false,
                fallback_reason: Some("onboarding review does not update finding history"),
            },
        };
    }

    let (path, mode, mut fallback_reason) = history_path(Path::new(project_root));
    let previous = read_history(&path);
    if path.exists() && previous.is_none() {
        fallback_reason = Some("history cache was unreadable; starting fresh");
    }
    let loaded = previous.is_some();
    let previous_by_fingerprint = previous
        .unwrap_or_default()
        .into_iter()
        .map(|snapshot| (snapshot.fingerprint.clone(), snapshot))
        .collect::<BTreeMap<_, _>>();
    let mut current_fingerprints = BTreeSet::new();

    for finding in findings.iter_mut() {
        current_fingerprints.insert(finding.fingerprint.clone());
        finding.state = match previous_by_fingerprint.get(&finding.fingerprint) {
            None => "new",
            Some(previous) if finding_worsened(previous, finding) => "worsened",
            Some(_) => "unchanged",
        };
    }

    for previous in previous_by_fingerprint.values() {
        if !current_fingerprints.contains(&previous.fingerprint) {
            findings.push(resolved_finding(previous));
        }
    }

    let snapshots = findings
        .iter()
        .filter(|finding| finding.state != "resolved")
        .map(FindingSnapshot::from)
        .collect::<Vec<_>>();
    let written = write_history(&path, snapshots);

    FindingHistorySummary {
        new: count_state(findings, "new"),
        worsened: count_state(findings, "worsened"),
        unchanged: count_state(findings, "unchanged"),
        resolved: count_state(findings, "resolved"),
        cache: FindingHistoryCache {
            mode,
            path: path.to_string_lossy().replace('\\', "/"),
            loaded,
            written,
            fallback_reason,
        },
    }
}

fn finding_fingerprint(finding: &ProjectReviewFinding) -> String {
    let identity_detail = if finding.source == "doctor.blocking_violations" {
        let (path, message) = finding
            .detail
            .split_once(": ")
            .unwrap_or((finding.detail.as_str(), ""));
        format!("{path}: {}", normalize_signal_numbers(message))
    } else {
        String::new()
    };
    digest(&format!(
        "{}\0{}\0{}",
        finding.source, finding.id, identity_detail
    ))
}

fn finding_worsened(previous: &FindingSnapshot, current: &ProjectReviewFinding) -> bool {
    severity_rank(current.severity) > severity_rank(&previous.severity)
        || signal_pressure(current).is_some_and(|pressure| {
            previous
                .pressure
                .is_some_and(|previous| pressure > previous)
        })
}

fn severity_rank(severity: &str) -> usize {
    match severity {
        "informational" => 0,
        "inactive" => 1,
        "advisory" => 2,
        "blocking" => 3,
        _ => 0,
    }
}

fn resolved_finding(previous: &FindingSnapshot) -> ProjectReviewFinding {
    ProjectReviewFinding {
        id: previous.id.clone(),
        fingerprint: previous.fingerprint.clone(),
        state: "resolved",
        category: "history",
        severity: "informational",
        action_kind: "informational",
        title: format!("Resolved: {}", previous.title),
        detail: previous.detail.clone(),
        command: "assura review --format json .",
        source: "review.finding_history",
    }
}

fn history_path(project_root: &Path) -> (PathBuf, &'static str, Option<&'static str>) {
    if let Some(path) = git_history_path(project_root) {
        return (path, "git-worktree", None);
    }
    let key = digest(&project_root.to_string_lossy());
    (
        std::env::temp_dir()
            .join("assura")
            .join("review-history")
            .join(format!("{key}.json")),
        "temporary",
        Some("Git worktree metadata is unavailable"),
    )
}

fn git_history_path(project_root: &Path) -> Option<PathBuf> {
    let project_key = digest(
        &project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf())
            .to_string_lossy(),
    );
    let git_path = format!("assura/review/{project_key}.json");
    let output = Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--git-path", &git_path])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    Some(if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    })
}

fn read_history(path: &Path) -> Option<Vec<FindingSnapshot>> {
    let bytes = fs::read(path).ok()?;
    let history: FindingHistoryFile = serde_json::from_slice(&bytes).ok()?;
    (history.schema == HISTORY_SCHEMA).then_some(history.findings)
}

fn write_history(path: &Path, findings: Vec<FindingSnapshot>) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    if fs::create_dir_all(parent).is_err() {
        return false;
    }
    set_private_permissions(parent, true);
    let history = FindingHistoryFile {
        schema: HISTORY_SCHEMA.to_string(),
        findings,
    };
    let Some(bytes) = serde_json::to_vec_pretty(&history).ok() else {
        return false;
    };
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    if fs::write(&temporary, bytes).is_err() {
        return false;
    }
    set_private_permissions(&temporary, false);
    #[cfg(windows)]
    if path.exists() && fs::remove_file(path).is_err() {
        let _ = fs::remove_file(&temporary);
        return false;
    }
    fs::rename(&temporary, path).is_ok()
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn normalize_signal_numbers(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut in_number = false;
    let mut quote = None;
    for character in value.chars() {
        if matches!(character, '\'' | '"' | '`') {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
            in_number = false;
            normalized.push(character);
        } else if character.is_ascii_digit() && quote.is_none() {
            if !in_number {
                normalized.push('#');
            }
            in_number = true;
        } else {
            in_number = false;
            normalized.push(character);
        }
    }
    normalized
}

fn signal_pressure(finding: &ProjectReviewFinding) -> Option<usize> {
    let message = finding
        .detail
        .split_once(": ")
        .map(|(_, message)| message)
        .unwrap_or(&finding.detail);
    let unquoted = strip_quoted(message);
    let numbers = unquoted
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<usize>().ok())
        .collect::<Vec<_>>();
    let actual = *numbers.first()?;
    match finding.id.strip_prefix("blocking:")? {
        "max_lines" | "max_size" => numbers
            .last()
            .copied()
            .map(|maximum| actual.saturating_sub(maximum)),
        "exists_count" => {
            let minimum = *numbers.get(1)?;
            let maximum = numbers.get(2).copied().unwrap_or(minimum);
            Some(if actual < minimum {
                minimum - actual
            } else {
                actual.saturating_sub(maximum)
            })
        }
        _ => None,
    }
}

fn strip_quoted(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut quote = None;
    for character in value.chars() {
        if matches!(character, '\'' | '"' | '`') {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
            output.push(' ');
        } else if quote.is_none() {
            output.push(character);
        } else {
            output.push(' ');
        }
    }
    output
}

#[cfg(unix)]
fn set_private_permissions(path: &Path, directory: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if directory { 0o700 } else { 0o600 };
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path, _directory: bool) {}

fn count_state(findings: &[ProjectReviewFinding], state: &str) -> usize {
    findings
        .iter()
        .filter(|finding| finding.state == state)
        .count()
}

impl From<&ProjectReviewFinding> for FindingSnapshot {
    fn from(finding: &ProjectReviewFinding) -> Self {
        Self {
            fingerprint: finding.fingerprint.clone(),
            id: finding.id.clone(),
            category: finding.category.to_string(),
            severity: finding.severity.to_string(),
            action_kind: finding.action_kind.to_string(),
            title: finding.title.clone(),
            detail: finding.detail.clone(),
            detail_hash: digest(&finding.detail),
            pressure: signal_pressure(finding),
            command: finding.command.to_string(),
            source: finding.source.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(detail: &str, severity: &'static str) -> ProjectReviewFinding {
        ProjectReviewFinding {
            id: "blocking:max_lines".to_string(),
            fingerprint: String::new(),
            state: "new",
            category: "structure",
            severity,
            action_kind: "fix-now",
            title: "Line count threshold exceeded".to_string(),
            detail: detail.to_string(),
            command: "assura explain src/file1.rs --format json",
            source: "doctor.blocking_violations",
        }
    }

    #[test]
    fn fingerprints_preserve_exact_paths_while_normalizing_message_counts() {
        let first = finding("src/file1.rs: found 412 lines; maximum is 300", "blocking");
        let same_signal = finding("src/file1.rs: found 430 lines; maximum is 300", "blocking");
        let other_path = finding("src/file2.rs: found 412 lines; maximum is 300", "blocking");

        assert_eq!(
            finding_fingerprint(&first),
            finding_fingerprint(&same_signal)
        );
        assert_ne!(
            finding_fingerprint(&first),
            finding_fingerprint(&other_path)
        );

        let mut guide_one = finding(
            "docs: Directory '.' has 0 files matching 'guide1.md', expected 1",
            "blocking",
        );
        guide_one.id = "blocking:exists_count".to_string();
        let mut guide_two = finding(
            "docs: Directory '.' has 0 files matching 'guide2.md', expected 1",
            "blocking",
        );
        guide_two.id = "blocking:exists_count".to_string();
        assert_ne!(
            finding_fingerprint(&guide_one),
            finding_fingerprint(&guide_two)
        );
    }

    #[test]
    fn only_increased_severity_or_numeric_pressure_is_worsened() {
        let previous = FindingSnapshot::from(&finding(
            "src/file1.rs: found 412 lines; maximum is 300",
            "blocking",
        ));
        let improved = finding("src/file1.rs: found 350 lines; maximum is 300", "blocking");
        let increased = finding("src/file1.rs: found 450 lines; maximum is 300", "blocking");

        assert!(!finding_worsened(&previous, &improved));
        assert!(finding_worsened(&previous, &increased));
    }

    #[test]
    fn exists_count_pressure_understands_both_minimum_and_maximum_drift() {
        let mut previous = finding(
            "docs: Directory '.' has 4 files matching '*.md', expected 5",
            "blocking",
        );
        previous.id = "blocking:exists_count".to_string();
        let previous = FindingSnapshot::from(&previous);
        let mut worse = finding(
            "docs: Directory '.' has 0 files matching '*.md', expected 5",
            "blocking",
        );
        worse.id = "blocking:exists_count".to_string();
        let mut improved = finding(
            "docs: Directory '.' has 5 files matching '*.md', expected 5",
            "blocking",
        );
        improved.id = "blocking:exists_count".to_string();

        assert!(finding_worsened(&previous, &worse));
        assert!(!finding_worsened(&previous, &improved));
    }

    #[test]
    fn history_file_can_be_replaced_after_the_first_write() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("history.json");
        let snapshot = FindingSnapshot::from(&finding(
            "src/file.rs: found 412 lines; maximum is 300",
            "blocking",
        ));

        assert!(write_history(&path, vec![snapshot.clone()]));
        assert!(write_history(&path, vec![snapshot]));
        assert_eq!(read_history(&path).expect("history").len(), 1);
    }
}
