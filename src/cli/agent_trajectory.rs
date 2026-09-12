//! Cached, bounded Git trajectory facts for explicit agent inspection.

#[path = "agent_trajectory_git.rs"]
mod git;
#[path = "agent_trajectory_snapshot.rs"]
mod snapshot;

use crate::config::config::AgentFeedbackConfig;
use git::{CategoryStats, CollectionResult, GitInput};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SNAPSHOT_SCHEMA: &str = "assura.agent-trajectory.v1";
const CLASSIFICATION_VERSION: &str = "paths-v1";

/// A bounded, cached snapshot of locally observable Git facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TrajectorySnapshot {
    schema: String,
    generation: String,
    #[serde(default)]
    configuration: String,
    repository: RepositoryIdentity,
    worktree: WorktreeIdentity,
    integration: IntegrationIdentity,
    captured_at: i64,
    pub(super) coverage: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    coverage_reasons: Vec<String>,
    window: TrajectoryWindow,
    #[serde(skip_serializing_if = "Option::is_none")]
    integrated: Option<HistoryMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pending: Option<PendingMetrics>,
    pub(super) freshness: Freshness,
    cache: CacheInfo,
}

impl TrajectorySnapshot {
    pub(super) fn coverage(&self) -> &str {
        &self.coverage
    }

    pub(super) fn age_seconds(&self, now: i64) -> i64 {
        now.saturating_sub(self.captured_at).max(0)
    }

    pub(super) fn captured_at(&self) -> i64 {
        self.captured_at
    }

    pub(super) fn has_pending_work(&self) -> bool {
        let Some(pending) = &self.pending else {
            return false;
        };
        pending.commits.is_some_and(|value| value > 0)
            || pending.files.is_some_and(|value| value > 0)
            || pending.dirty_files.is_some_and(|value| value > 0)
            || [
                &pending.source,
                &pending.tests,
                &pending.coordination,
                &pending.generated,
                &pending.other,
            ]
            .iter()
            .any(|category| category.files.is_some_and(|value| value > 0))
    }

    pub(super) fn pending_changed_lines(&self) -> Option<u64> {
        let pending = self.pending.as_ref()?;
        let categories = [
            &pending.source,
            &pending.tests,
            &pending.coordination,
            &pending.generated,
            &pending.other,
        ];
        let mut total = 0u64;
        for category in categories {
            total = total
                .checked_add(category.additions?)
                .and_then(|value| value.checked_add(category.deletions?))?;
        }
        Some(total)
    }

    pub(super) fn coordination_only_commits(&self, window: u64) -> Option<u64> {
        if self.window.kind != "commits" || self.window.value != window {
            return None;
        }
        let history = self.integrated.as_ref()?;
        history
            .complete_window
            .then_some(history.coordination_only_commits?)
    }

    pub(super) fn compact_line_for(
        &self,
        episode_started_at: Option<i64>,
        max_bytes: usize,
        metrics: &[String],
    ) -> String {
        let mut fields = vec![format!(
            "base={}{}",
            self.integration.requested_ref,
            self.integration
                .sha
                .as_ref()
                .map(|_| "(local)")
                .unwrap_or("(?)")
        )];
        fields.push(format!(
            "{}{}",
            self.window.value,
            if self.window.kind == "minutes" {
                "m"
            } else {
                "c"
            }
        ));
        if metrics.iter().any(|metric| metric == "integrated_commits") {
            if let Some(commits) = self.integrated.as_ref().and_then(|value| value.commits) {
                fields.push(format!("commits={commits}"));
            }
        }
        if let Some(integrated) = &self.integrated {
            for (metric, label, category) in [
                ("source_lines", "src", &integrated.source),
                ("test_lines", "test", &integrated.tests),
                ("coordination_lines", "coord", &integrated.coordination),
                ("generated_lines", "gen", &integrated.generated),
            ] {
                if metrics.iter().any(|value| value == metric) {
                    if let Some((additions, deletions)) = known_lines(category) {
                        fields.push(format!("{label}=+{additions}/-{deletions}"));
                    }
                }
            }
        }
        if let Some(pending) = &self.pending {
            if metrics.iter().any(|metric| metric == "pending_age") {
                if let Some(started) = episode_started_at {
                    let age = self.captured_at.saturating_sub(started).max(0) / 60;
                    fields.push(format!("pending={age}m"));
                }
            }
            if metrics.iter().any(|metric| metric == "pending_commits") {
                if let Some(commits) = pending.commits {
                    fields.push(format!("pending_commits={commits}"));
                }
            }
            if metrics.iter().any(|metric| metric == "pending_lines") {
                let categories = [
                    &pending.source,
                    &pending.tests,
                    &pending.coordination,
                    &pending.generated,
                    &pending.other,
                ];
                let totals = categories.into_iter().try_fold(
                    (0u64, 0u64),
                    |(additions, deletions), category| {
                        let (next_additions, next_deletions) = known_lines(category)?;
                        Some((
                            additions.saturating_add(next_additions),
                            deletions.saturating_add(next_deletions),
                        ))
                    },
                );
                if let Some((additions, deletions)) = totals {
                    fields.push(format!("lines=+{additions}/-{deletions}"));
                }
            }
            if metrics.iter().any(|metric| metric == "dirty_files") {
                if let Some(dirty) = pending.dirty_files {
                    fields.push(format!("dirty={dirty}"));
                }
            }
        }
        bound_line(format!("assura: {}", fields.join(" ")), max_bytes)
    }
}

fn known_lines(category: &CategoryMetrics) -> Option<(u64, u64)> {
    category.additions.zip(category.deletions)
}

fn bound_line(value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    String::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RepositoryIdentity {
    root: String,
    common_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorktreeIdentity {
    root: String,
    git_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    head_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntegrationIdentity {
    requested_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrajectoryWindow {
    kind: String,
    value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryMetrics {
    commits: Option<u64>,
    #[serde(default)]
    coordination_only_commits: Option<u64>,
    #[serde(default)]
    complete_window: bool,
    source: CategoryMetrics,
    tests: CategoryMetrics,
    coordination: CategoryMetrics,
    generated: CategoryMetrics,
    other: CategoryMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingMetrics {
    commits: Option<u64>,
    files: Option<u64>,
    dirty_files: Option<u64>,
    source: CategoryMetrics,
    tests: CategoryMetrics,
    coordination: CategoryMetrics,
    generated: CategoryMetrics,
    other: CategoryMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryMetrics {
    files: Option<u64>,
    additions: Option<u64>,
    deletions: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Freshness {
    pub(super) snapshot: String,
    pub(super) integration_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheInfo {
    status: String,
    source: String,
}

pub(super) fn inspect_with_config(
    project_root: &Path,
    config: Option<&AgentFeedbackConfig>,
) -> Result<TrajectorySnapshot, String> {
    let now = current_time()?;
    inspect_at(project_root, now, config)
}

pub(super) fn cached_snapshot(
    project_root: &Path,
    config: &AgentFeedbackConfig,
) -> Option<TrajectorySnapshot> {
    let marker = project_root.join(".git");
    let git_dir = if marker.is_dir() {
        marker
    } else {
        let value = std::fs::read_to_string(marker).ok()?;
        let value = value.strip_prefix("gitdir: ")?.trim();
        let path = Path::new(value);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            project_root.join(path)
        }
    };
    let common_dir = if git_dir.file_name().and_then(|name| name.to_str()) == Some(".git") {
        git_dir.clone()
    } else {
        git_dir.parent()?.parent()?.to_path_buf()
    };
    let root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let expected_configuration = collector_configuration_key(Some(config));
    snapshot::read_latest_for_worktree(&common_dir, &root)
        .snapshot
        .filter(|snapshot| snapshot.configuration == expected_configuration)
}

fn collector_configuration_key(config: Option<&AgentFeedbackConfig>) -> String {
    config
        .map(|config| {
            serde_json::to_string(&(config.collection.max_commits, &config.trajectory))
                .unwrap_or_default()
        })
        .unwrap_or_else(|| "unconfigured".to_string())
}

fn inspect_at(
    project_root: &Path,
    now: i64,
    config: Option<&AgentFeedbackConfig>,
) -> Result<TrajectorySnapshot, String> {
    let input = git::discover(
        project_root,
        now,
        config.map(|value| &value.trajectory),
        config.map_or(500, |value| u64::from(value.collection.max_commits)),
    )?;
    let generation = git::generation(&input, CLASSIFICATION_VERSION, now);
    let cache_key = snapshot::CacheKey::new(&input, CLASSIFICATION_VERSION);
    let cache_read = snapshot::read(&cache_key);

    if let Some(previous) = cache_read.snapshot.as_ref() {
        if previous.generation == generation && previous.coverage == "complete" {
            let mut cached = previous.clone();
            cached.freshness = Freshness {
                snapshot: "cache_hit".to_string(),
                integration_ref: "unchanged_local".to_string(),
            };
            cached.cache = CacheInfo {
                status: "hit".to_string(),
                source: cache_read.source.to_string(),
            };
            return Ok(cached);
        }
    }

    let collection = git::collect(&input, now);
    let cache_source = if cache_read.snapshot.is_some() {
        "stale"
    } else {
        cache_read.source
    };
    let mut snapshot = build_snapshot(
        &input,
        generation,
        now,
        collection,
        cache_source,
        &cache_read,
        config,
    );
    if super::agent_nudge_delivery::refresh_deadline_expired() {
        return Err("trajectory refresh deadline exceeded".to_string());
    }
    if let Err(error) = snapshot::write(&cache_key, &snapshot) {
        snapshot.cache = CacheInfo {
            status: "refreshed".to_string(),
            source: "write_failed".to_string(),
        };
        snapshot
            .coverage_reasons
            .push("cache_write_failed".to_string());
        snapshot.coverage = "partial".to_string();
        let _ = error;
    }
    Ok(snapshot)
}

fn current_time() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))
}

fn build_snapshot(
    input: &GitInput,
    generation: String,
    now: i64,
    collection: CollectionResult,
    cache_source: &str,
    cache_read: &snapshot::CacheRead,
    config: Option<&AgentFeedbackConfig>,
) -> TrajectorySnapshot {
    let mut reasons = collection.reasons;
    if input.status_truncated {
        reasons.push("status_output_truncated".to_string());
    }
    if input.status_unavailable {
        reasons.push("status_unavailable".to_string());
    }
    if input.status_timed_out {
        reasons.push("status_timeout".to_string());
    }
    if let Some(previous) = cache_read.snapshot.as_ref() {
        if previous.captured_at > now {
            reasons.push("clock_rollback".to_string());
        }
        if previous.integration.sha != input.integration_sha
            && previous.integration.resolved_ref == input.integration_ref
        {
            reasons.push("integration_ref_changed".to_string());
        }
    }
    reasons.sort();
    reasons.dedup();

    let coverage = if !input.available {
        "unknown"
    } else if reasons.is_empty() {
        "complete"
    } else {
        "partial"
    };
    let integration_ref = if input.integration_sha.is_none() {
        "missing"
    } else if reasons
        .iter()
        .any(|reason| reason == "integration_ref_changed")
    {
        "changed_local"
    } else {
        "known_local"
    };

    TrajectorySnapshot {
        schema: SNAPSHOT_SCHEMA.to_string(),
        generation,
        configuration: collector_configuration_key(config),
        repository: RepositoryIdentity {
            root: input.repo_root.display().to_string(),
            common_dir: input.common_dir.display().to_string(),
        },
        worktree: WorktreeIdentity {
            root: input.worktree_root.display().to_string(),
            git_dir: input.git_dir.display().to_string(),
            branch: input.branch.clone(),
            head_sha: input.head_sha.clone(),
        },
        integration: IntegrationIdentity {
            requested_ref: input.requested_integration_ref.clone(),
            resolved_ref: input.integration_ref.clone(),
            sha: input.integration_sha.clone(),
        },
        captured_at: now,
        coverage: coverage.to_string(),
        coverage_reasons: reasons,
        window: TrajectoryWindow {
            kind: input.window.kind().to_string(),
            value: input.window.value(),
        },
        integrated: collection.history.map(history_metrics),
        pending: collection.pending.map(pending_metrics),
        freshness: Freshness {
            snapshot: "refreshed".to_string(),
            integration_ref: integration_ref.to_string(),
        },
        cache: CacheInfo {
            status: "refreshed".to_string(),
            source: cache_source.to_string(),
        },
    }
}

fn history_metrics(history: git::HistoryResult) -> HistoryMetrics {
    HistoryMetrics {
        commits: Some(history.commits),
        coordination_only_commits: history.coordination_only_commits,
        complete_window: history.complete_window,
        source: category_metrics(history.categories[0].clone()),
        tests: category_metrics(history.categories[1].clone()),
        coordination: category_metrics(history.categories[2].clone()),
        generated: category_metrics(history.categories[3].clone()),
        other: category_metrics(history.categories[4].clone()),
    }
}

fn pending_metrics(pending: git::PendingResult) -> PendingMetrics {
    PendingMetrics {
        commits: pending.commits,
        files: Some(pending.files),
        dirty_files: pending.dirty_files,
        source: category_metrics(pending.categories[0].clone()),
        tests: category_metrics(pending.categories[1].clone()),
        coordination: category_metrics(pending.categories[2].clone()),
        generated: category_metrics(pending.categories[3].clone()),
        other: category_metrics(pending.categories[4].clone()),
    }
}

fn category_metrics(stats: CategoryStats) -> CategoryMetrics {
    CategoryMetrics {
        files: Some(stats.files),
        additions: (!stats.unknown_lines).then_some(stats.additions),
        deletions: (!stats.unknown_lines).then_some(stats.deletions),
    }
}

#[cfg(test)]
mod tests {
    use super::git::{collect_history, Window};
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn trajectory_windows_have_stable_public_labels() {
        assert_eq!(Window::Minutes(30).kind(), "minutes");
        assert_eq!(Window::Minutes(30).value(), 30);
        assert_eq!(Window::Minutes(30).cache_bucket(1799), "minute:0");
        assert_eq!(Window::Minutes(30).cache_bucket(1800), "minute:1");
        assert_eq!(Window::Commits(20).kind(), "commits");
        assert_eq!(Window::Commits(20).value(), 20);
        assert_eq!(Window::Commits(20).cache_bucket(119), "commits");
    }

    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "Assura Test")
            .env("GIT_AUTHOR_EMAIL", "assura-test@example.com")
            .env("GIT_COMMITTER_NAME", "Assura Test")
            .env("GIT_COMMITTER_EMAIL", "assura-test@example.com")
            .output()
            .expect("git test command runs");
        assert!(output.status.success(), "git {:?}", args);
    }

    #[test]
    fn history_collection_supports_minute_and_commit_windows() {
        let root = tempfile::tempdir().expect("history fixture");
        git(root.path(), &["init"]);
        git(root.path(), &["branch", "-M", "master"]);
        fs::write(root.path().join("history.txt"), "one\n").expect("write history");
        git(root.path(), &["add", "."]);
        git(root.path(), &["commit", "-m", "one"]);
        fs::write(root.path().join("history.txt"), "one\ntwo\n").expect("write history");
        git(root.path(), &["add", "."]);
        git(root.path(), &["commit", "-m", "two"]);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs() as i64
            + 60;
        let (minutes, _) = collect_history(root.path(), "master", Window::Minutes(30), now)
            .expect("minute history is available");
        let (commits, _) = collect_history(root.path(), "master", Window::Commits(1), now)
            .expect("commit history is available");
        assert_eq!(minutes.commits, 2);
        assert_eq!(commits.commits, 1);
    }

    #[test]
    fn minute_cache_refreshes_after_the_window_bucket_changes() {
        let root = tempfile::tempdir().expect("cache fixture");
        git(root.path(), &["init"]);
        git(root.path(), &["branch", "-M", "master"]);
        fs::write(root.path().join("history.txt"), "one\n").expect("write history");
        git(root.path(), &["add", "."]);
        git(root.path(), &["commit", "-m", "one"]);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs() as i64;
        let first = super::inspect_at(root.path(), now, None).expect("first inspect");
        let second =
            super::inspect_at(root.path(), now + 30 * 60, None).expect("next bucket inspect");
        assert_eq!(first.cache.status, "refreshed");
        assert_eq!(second.cache.status, "refreshed");
        assert_eq!(second.freshness.snapshot, "refreshed");
    }
}
