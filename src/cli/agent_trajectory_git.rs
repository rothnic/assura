//! Git subprocesses and bounded trajectory fact collection.

#[path = "agent_trajectory_git_exec.rs"]
mod exec;

use crate::config::config::AgentFeedbackTrajectoryConfig;
use glob::Pattern;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Instant;

use exec::{run_git, GitOutput};

const MAX_HISTORY_BYTES: usize = 4 * 1024 * 1024;
const MAX_STATUS_BYTES: usize = 256 * 1024;
const MAX_COMMITS: u64 = 500;

#[derive(Debug, Clone)]
pub(super) struct GitInput {
    pub(super) available: bool,
    pub(super) repo_root: PathBuf,
    pub(super) common_dir: PathBuf,
    pub(super) git_dir: PathBuf,
    pub(super) worktree_root: PathBuf,
    pub(super) branch: Option<String>,
    pub(super) head_sha: Option<String>,
    pub(super) requested_integration_ref: String,
    pub(super) integration_ref: Option<String>,
    pub(super) integration_sha: Option<String>,
    pub(super) merge_base: Option<String>,
    pub(super) shallow: bool,
    pub(super) status_truncated: bool,
    pub(super) status_unavailable: bool,
    pub(super) status_timed_out: bool,
    pub(super) status_files: Option<u64>,
    pub(super) untracked_paths: Vec<String>,
    pub(super) status_digest: String,
    pub(super) window: Window,
    pub(super) max_commits: u64,
    pub(super) classification: PathClassification,
    pub(super) deadline: Option<Instant>,
}

#[derive(Debug, Clone)]
pub(super) struct PathClassification {
    source: Vec<String>,
    tests: Vec<String>,
    coordination: Vec<String>,
    generated: Vec<String>,
}

impl Default for PathClassification {
    fn default() -> Self {
        Self {
            source: ["src/**", "xtask/src/**", "website/src/**"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            tests: ["tests/**"].into_iter().map(str::to_string).collect(),
            coordination: [".trellis/**", ".agents/**", ".codex/**", "docs/goals/**"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            generated: ["target/**", "dist/**"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }
}

impl PathClassification {
    fn from_config(config: &AgentFeedbackTrajectoryConfig) -> Self {
        Self {
            source: config.source_paths.clone(),
            tests: config.test_paths.clone(),
            coordination: config.coordination_paths.clone(),
            generated: config.generated_paths.clone(),
        }
    }

    fn key(&self) -> String {
        format!(
            "s={:?};t={:?};c={:?};g={:?}",
            self.source, self.tests, self.coordination, self.generated
        )
    }
}

// allow-reason: commit-window parsing shares the bounded collector's public shape with minute-window parsing.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(super) enum Window {
    Minutes(u64),
    Commits(u64),
}

impl Window {
    pub(super) fn kind(self) -> &'static str {
        match self {
            Self::Minutes(_) => "minutes",
            Self::Commits(_) => "commits",
        }
    }

    pub(super) fn value(self) -> u64 {
        match self {
            Self::Minutes(value) | Self::Commits(value) => value,
        }
    }

    pub(super) fn cache_bucket(self, now: i64) -> String {
        match self {
            Self::Minutes(minutes) => {
                format!("minute:{}", now.div_euclid((minutes.max(1) * 60) as i64))
            }
            Self::Commits(_) => "commits".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct CollectionResult {
    pub(super) history: Option<HistoryResult>,
    pub(super) pending: Option<PendingResult>,
    pub(super) reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct HistoryResult {
    pub(super) commits: u64,
    pub(super) categories: [CategoryStats; 5],
    pub(super) coordination_only_commits: Option<u64>,
    pub(super) complete_window: bool,
}

#[derive(Debug, Clone)]
pub(super) struct PendingResult {
    pub(super) commits: Option<u64>,
    pub(super) files: u64,
    pub(super) dirty_files: Option<u64>,
    pub(super) categories: [CategoryStats; 5],
}

#[derive(Debug, Clone, Default)]
pub(super) struct CategoryStats {
    pub(super) files: u64,
    pub(super) additions: u64,
    pub(super) deletions: u64,
    pub(super) unknown_lines: bool,
}

pub(super) fn discover(
    project_root: &Path,
    now: i64,
    trajectory: Option<&AgentFeedbackTrajectoryConfig>,
    max_commits: u64,
    deadline: Option<Instant>,
) -> Result<GitInput, String> {
    let classification = trajectory
        .map(PathClassification::from_config)
        .unwrap_or_default();
    let window = trajectory
        .and_then(|config| {
            config
                .window
                .minutes
                .map(Window::Minutes)
                .or_else(|| config.window.commits.map(Window::Commits))
        })
        .unwrap_or(Window::Minutes(30));
    let requested_integration_ref = trajectory
        .map(|config| config.integration_ref.clone())
        .unwrap_or_else(|| "origin/master".to_string());
    let worktree_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let Some(repo_text) = git_text(
        project_root,
        &["rev-parse", "--show-toplevel"],
        8 * 1024,
        deadline,
    ) else {
        return Ok(unavailable(
            worktree_root,
            now,
            requested_integration_ref,
            window,
            classification,
            max_commits,
            deadline,
        ));
    };
    let repo_root = PathBuf::from(repo_text.trim())
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(repo_text.trim()));
    let common_dir = resolve_git_path(
        &repo_root,
        git_text(
            &repo_root,
            &["rev-parse", "--git-common-dir"],
            8 * 1024,
            deadline,
        ),
    );
    let git_dir = resolve_git_path(
        &repo_root,
        git_text(&repo_root, &["rev-parse", "--git-dir"], 8 * 1024, deadline),
    );
    let branch = git_text(
        &repo_root,
        &["symbolic-ref", "--quiet", "--short", "HEAD"],
        8 * 1024,
        deadline,
    )
    .map(|value| value.trim().to_string());
    let head_sha = git_text(
        &repo_root,
        &["rev-parse", "--verify", "HEAD"],
        8 * 1024,
        deadline,
    )
    .map(|value| value.trim().to_string());
    let (integration_ref, integration_sha) =
        resolve_integration_ref(&repo_root, &requested_integration_ref, deadline);
    let merge_base = integration_ref
        .as_deref()
        .and_then(|reference| {
            git_text(
                &repo_root,
                &["merge-base", "HEAD", reference],
                8 * 1024,
                deadline,
            )
        })
        .map(|value| value.trim().to_string());
    let shallow = git_text(
        &repo_root,
        &["rev-parse", "--is-shallow-repository"],
        8 * 1024,
        deadline,
    )
    .is_some_and(|value| value.trim() == "true");
    let status = status_facts(&repo_root, deadline);

    Ok(GitInput {
        available: true,
        repo_root,
        common_dir,
        git_dir,
        worktree_root,
        branch,
        head_sha,
        requested_integration_ref,
        integration_ref,
        integration_sha,
        merge_base,
        shallow,
        status_truncated: status.truncated,
        status_unavailable: status.unavailable,
        status_timed_out: status.timed_out,
        status_files: status.files,
        untracked_paths: status.untracked_paths,
        status_digest: status.digest,
        window,
        max_commits: max_commits.clamp(1, MAX_COMMITS),
        classification,
        deadline,
    })
}

pub(super) fn unavailable(
    worktree_root: PathBuf,
    _now: i64,
    requested_integration_ref: String,
    window: Window,
    classification: PathClassification,
    max_commits: u64,
    deadline: Option<Instant>,
) -> GitInput {
    GitInput {
        available: false,
        repo_root: worktree_root.clone(),
        common_dir: worktree_root.clone(),
        git_dir: worktree_root.clone(),
        worktree_root,
        branch: None,
        head_sha: None,
        requested_integration_ref,
        integration_ref: None,
        integration_sha: None,
        merge_base: None,
        shallow: false,
        status_truncated: false,
        status_unavailable: false,
        status_timed_out: false,
        status_files: None,
        untracked_paths: Vec::new(),
        status_digest: String::new(),
        window,
        max_commits: max_commits.clamp(1, MAX_COMMITS),
        classification,
        deadline,
    }
}

pub(super) fn generation(input: &GitInput, classification_version: &str, now: i64) -> String {
    digest(&format!(
        "{classification_version}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        input.repo_root.display(),
        input.git_dir.display(),
        input.branch.as_deref().unwrap_or(""),
        input.head_sha.as_deref().unwrap_or(""),
        input.requested_integration_ref,
        input.integration_ref.as_deref().unwrap_or(""),
        input.integration_sha.as_deref().unwrap_or(""),
        input.status_digest,
        input.status_truncated,
        input.status_unavailable,
        input.status_timed_out,
        input.status_files.map_or(-1, |files| files as i64),
        input.window.kind(),
        input.window.value(),
        input.window.cache_bucket(now),
        input.shallow,
        input.max_commits,
        input.classification.key()
    ))
}

pub(super) fn collect(input: &GitInput, now: i64) -> CollectionResult {
    if !input.available {
        return CollectionResult {
            reasons: vec!["not_git_repository".to_string()],
            ..CollectionResult::default()
        };
    }
    let mut reasons = Vec::new();
    if input.integration_ref.is_none() {
        reasons.push("integration_ref_missing".to_string());
    }
    if input.merge_base.is_none() {
        reasons.push("no_common_ancestor".to_string());
    }
    if input.shallow {
        reasons.push("shallow_history".to_string());
    }
    if input.status_truncated {
        reasons.push("status_output_truncated".to_string());
    }
    if input.status_unavailable {
        reasons.push("status_unavailable".to_string());
    }
    if input.status_timed_out {
        reasons.push("status_timeout".to_string());
    }

    let history = input.integration_ref.as_deref().and_then(|reference| {
        match collect_history_with_paths(
            &input.repo_root,
            reference,
            input.window,
            now,
            input.max_commits,
            &input.classification,
            input.deadline,
        ) {
            Ok((history, capped)) => {
                if capped {
                    reasons.push("history_cap".to_string());
                }
                Some(history)
            }
            Err(reason) => {
                reasons.push(reason);
                None
            }
        }
    });

    let pending = input.merge_base.as_deref().and_then(|merge_base| {
        match collect_pending(&input.repo_root, input, merge_base) {
            Ok(pending) => Some(pending),
            Err(reason) => {
                reasons.push(reason);
                None
            }
        }
    });

    CollectionResult {
        history,
        pending,
        reasons,
    }
}

#[cfg(test)]
pub(super) fn collect_history(
    repo_root: &Path,
    reference: &str,
    window: Window,
    now: i64,
) -> Result<(HistoryResult, bool), String> {
    collect_history_with_paths(
        repo_root,
        reference,
        window,
        now,
        MAX_COMMITS,
        &PathClassification::default(),
        None,
    )
}

fn collect_history_with_paths(
    repo_root: &Path,
    reference: &str,
    window: Window,
    now: i64,
    configured_max_commits: u64,
    classification: &PathClassification,
    deadline: Option<Instant>,
) -> Result<(HistoryResult, bool), String> {
    let max_commits = match window {
        Window::Minutes(_) => configured_max_commits.min(MAX_COMMITS),
        Window::Commits(value) => value.min(configured_max_commits).min(MAX_COMMITS),
    };
    let mut args = vec![
        "log".to_string(),
        "--first-parent".to_string(),
        "--diff-merges=first-parent".to_string(),
        "--no-ext-diff".to_string(),
        "--numstat".to_string(),
        "--format=__ASSURA_COMMIT__%x09%H%x09%ct%x09%P".to_string(),
        "-n".to_string(),
        max_commits.saturating_add(1).to_string(),
    ];
    if let Window::Minutes(minutes) = window {
        args.push(format!(
            "--since=@{}",
            now.saturating_sub((minutes * 60) as i64)
        ));
    }
    args.push(reference.to_string());
    let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
    let text = match run_git(repo_root, &borrowed, MAX_HISTORY_BYTES, deadline) {
        GitOutput::Text(text) => text,
        GitOutput::Truncated => return Err("history_output_truncated".to_string()),
        GitOutput::Failed => return Err("history_unavailable".to_string()),
        GitOutput::TimedOut => return Err("history_timeout".to_string()),
    };
    let mut categories = std::array::from_fn(|_| CategoryStats::default());
    let mut commits = 0u64;
    let mut observed_commits = 0u64;
    let mut active_timestamp = None;
    let mut active_categories = [false; 5];
    let mut active_had_path = false;
    let mut coordination_only_commits = 0u64;
    for line in text.lines() {
        if let Some(fields) = line.strip_prefix("__ASSURA_COMMIT__\t") {
            if active_timestamp.is_some()
                && active_had_path
                && active_categories[2]
                && !active_categories[0]
                && !active_categories[1]
                && !active_categories[3]
                && !active_categories[4]
            {
                coordination_only_commits = coordination_only_commits.saturating_add(1);
            }
            observed_commits = observed_commits.saturating_add(1);
            if observed_commits > max_commits {
                active_timestamp = None;
                active_categories = [false; 5];
                active_had_path = false;
                continue;
            }
            commits = commits.saturating_add(1);
            active_timestamp = fields
                .split('\t')
                .nth(1)
                .and_then(|value| value.parse::<i64>().ok());
            active_categories = [false; 5];
            active_had_path = false;
            continue;
        }
        if active_timestamp.is_none() || !numstat_line(line) {
            continue;
        }
        if let Some((additions, deletions, path)) = parse_numstat(line) {
            let category = classify_path_with(path.as_str(), classification);
            active_categories[category] = true;
            active_had_path = true;
            add_category(&mut categories[category], additions, deletions);
        }
    }
    if active_timestamp.is_some()
        && active_had_path
        && active_categories[2]
        && !active_categories[0]
        && !active_categories[1]
        && !active_categories[3]
        && !active_categories[4]
    {
        coordination_only_commits = coordination_only_commits.saturating_add(1);
    }
    let capped = observed_commits > max_commits;
    let complete_window = matches!(window, Window::Commits(_)) && !capped && commits == max_commits;
    Ok((
        HistoryResult {
            commits,
            categories,
            coordination_only_commits: (!capped).then_some(coordination_only_commits),
            complete_window,
        },
        capped,
    ))
}

fn collect_pending(
    repo_root: &Path,
    input: &GitInput,
    merge_base: &str,
) -> Result<PendingResult, String> {
    let commits = input
        .integration_ref
        .as_deref()
        .and_then(|reference| {
            git_text(
                repo_root,
                &[
                    "rev-list",
                    "--first-parent",
                    "--count",
                    &format!("{reference}..HEAD"),
                ],
                8 * 1024,
                input.deadline,
            )
        })
        .and_then(|value| value.trim().parse::<u64>().ok());
    let same_integration_tree = input
        .integration_ref
        .as_deref()
        .and_then(|reference| trees_match(repo_root, reference, "HEAD", input.deadline));
    if same_integration_tree == Some(true)
        && input.status_files == Some(0)
        && input.untracked_paths.is_empty()
    {
        return Ok(PendingResult {
            commits: Some(0),
            files: 0,
            dirty_files: input.status_files,
            categories: std::array::from_fn(|_| CategoryStats::default()),
        });
    }
    let diff_base = if same_integration_tree == Some(true) {
        input.integration_ref.as_deref().unwrap_or(merge_base)
    } else {
        merge_base
    };
    let args = [
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "--find-renames",
        "--numstat",
        diff_base,
        "--",
    ];
    let text = match run_git(repo_root, &args, MAX_HISTORY_BYTES, input.deadline) {
        GitOutput::Text(text) => text,
        GitOutput::Truncated => return Err("pending_diff_output_truncated".to_string()),
        GitOutput::Failed => return Err("pending_diff_unavailable".to_string()),
        GitOutput::TimedOut => return Err("pending_diff_timeout".to_string()),
    };
    let mut categories = std::array::from_fn(|_| CategoryStats::default());
    let mut files = 0u64;
    for line in text.lines() {
        if let Some((additions, deletions, path)) = parse_numstat(line) {
            files = files.saturating_add(1);
            add_category(
                &mut categories[classify_path_with(path.as_str(), &input.classification)],
                additions,
                deletions,
            );
        }
    }
    for path in &input.untracked_paths {
        files = files.saturating_add(1);
        add_category(
            &mut categories[classify_path_with(path, &input.classification)],
            None,
            None,
        );
    }
    Ok(PendingResult {
        commits,
        files,
        dirty_files: input.status_files,
        categories,
    })
}

fn trees_match(
    repo_root: &Path,
    left: &str,
    right: &str,
    deadline: Option<Instant>,
) -> Option<bool> {
    match run_git(
        repo_root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--name-only",
            left,
            right,
            "--",
        ],
        MAX_STATUS_BYTES,
        deadline,
    ) {
        GitOutput::Text(text) => Some(text.trim().is_empty()),
        GitOutput::Failed | GitOutput::Truncated | GitOutput::TimedOut => None,
    }
}

fn add_category(category: &mut CategoryStats, additions: Option<u64>, deletions: Option<u64>) {
    category.files = category.files.saturating_add(1);
    match (additions, deletions) {
        (Some(additions), Some(deletions)) => {
            category.additions = category.additions.saturating_add(additions);
            category.deletions = category.deletions.saturating_add(deletions);
        }
        _ => category.unknown_lines = true,
    }
}

fn numstat_line(line: &str) -> bool {
    line.split('\t')
        .next()
        .is_some_and(|value| value == "-" || value.parse::<u64>().is_ok())
}

fn parse_numstat(line: &str) -> Option<(Option<u64>, Option<u64>, String)> {
    let mut fields = line.splitn(3, '\t');
    let additions = fields.next()?;
    let deletions = fields.next()?;
    let path = fields.next()?.trim();
    if !(additions == "-" || additions.parse::<u64>().is_ok())
        || !(deletions == "-" || deletions.parse::<u64>().is_ok())
        || path.is_empty()
    {
        return None;
    }
    Some((
        additions.parse::<u64>().ok(),
        deletions.parse::<u64>().ok(),
        normalize_path(path),
    ))
}

fn classify_path_with(path: &str, classification: &PathClassification) -> usize {
    let path = normalize_path(path);
    if matches_patterns(&path, &classification.generated) {
        3
    } else if matches_patterns(&path, &classification.tests) {
        1
    } else if matches_patterns(&path, &classification.coordination) {
        2
    } else if matches_patterns(&path, &classification.source) {
        0
    } else {
        4
    }
}

fn matches_patterns(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|pattern| pattern.matches(path))
            .unwrap_or(false)
    })
}

fn normalize_path(path: &str) -> String {
    let path = path.trim_matches('"').trim();
    let path = path
        .strip_prefix('{')
        .unwrap_or(path)
        .strip_suffix('}')
        .unwrap_or(path);
    path.rsplit_once(" => ")
        .map(|(_, destination)| destination)
        .unwrap_or(path)
        .replace('\\', "/")
}

struct StatusFacts {
    files: Option<u64>,
    truncated: bool,
    unavailable: bool,
    timed_out: bool,
    untracked_paths: Vec<String>,
    digest: String,
}

fn status_facts(repo_root: &Path, deadline: Option<Instant>) -> StatusFacts {
    let output = run_git(
        repo_root,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
        MAX_STATUS_BYTES,
        deadline,
    );
    let (text, truncated, files_available, unavailable, timed_out) = match output {
        GitOutput::Text(text) => (text, false, true, false, false),
        GitOutput::Truncated => (String::new(), true, false, false, false),
        GitOutput::Failed => (String::new(), false, false, true, false),
        GitOutput::TimedOut => (String::new(), false, false, true, true),
    };
    let relevant_lines = text
        .lines()
        .filter(|line| !line.trim().is_empty() && !is_runtime_status_path(line))
        .collect::<Vec<_>>();
    StatusFacts {
        files: files_available.then_some(relevant_lines.len() as u64),
        truncated,
        unavailable,
        timed_out,
        untracked_paths: relevant_lines
            .iter()
            .filter_map(|line| line.strip_prefix("?? "))
            .map(normalize_path)
            .collect(),
        digest: digest(&relevant_lines.join("\n")),
    }
}

fn is_runtime_status_path(line: &str) -> bool {
    let path = line.get(3..).unwrap_or(line).trim();
    let path = path
        .rsplit_once(" -> ")
        .map(|(_, destination)| destination)
        .unwrap_or(path)
        .trim_matches('"');
    path == ".assura/agent-sessions" || path.starts_with(".assura/agent-sessions/")
}

fn resolve_integration_ref(
    repo_root: &Path,
    requested: &str,
    deadline: Option<Instant>,
) -> (Option<String>, Option<String>) {
    let candidates = if requested == "origin/master" {
        vec![requested, "origin/main", "master", "main"]
    } else {
        vec![requested]
    };
    for candidate in candidates {
        if let Some(sha) = git_text(
            repo_root,
            &["rev-parse", "--verify", "--quiet", candidate],
            8 * 1024,
            deadline,
        ) {
            return (Some(candidate.to_string()), Some(sha.trim().to_string()));
        }
    }
    (None, None)
}

fn resolve_git_path(repo_root: &Path, value: Option<String>) -> PathBuf {
    let path = value
        .map(|value| PathBuf::from(value.trim()))
        .unwrap_or_else(|| repo_root.join(".git"));
    let path = if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    };
    path.canonicalize().unwrap_or(path)
}

fn git_text(
    repo_root: &Path,
    args: &[&str],
    limit: usize,
    deadline: Option<Instant>,
) -> Option<String> {
    match run_git(repo_root, args, limit, deadline) {
        GitOutput::Text(text) => Some(text),
        GitOutput::Failed | GitOutput::Truncated | GitOutput::TimedOut => None,
    }
}

fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
