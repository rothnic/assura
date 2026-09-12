//! Cached, bounded Git trajectory facts for explicit agent inspection.

#[path = "agent_trajectory_git.rs"]
mod git;
#[path = "agent_trajectory_snapshot.rs"]
mod snapshot;

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

/// Collect trajectory facts for an explicit, bounded inspect request.
pub(super) fn inspect(project_root: &Path) -> Result<TrajectorySnapshot, String> {
    let now = current_time()?;
    inspect_at(project_root, now)
}

fn inspect_at(project_root: &Path, now: i64) -> Result<TrajectorySnapshot, String> {
    let input = git::discover(project_root, now)?;
    let generation = git::generation(&input, CLASSIFICATION_VERSION);
    let cache_key = snapshot::CacheKey::new(&input, CLASSIFICATION_VERSION);
    let cache_read = snapshot::read(&cache_key);

    if let Some(previous) = cache_read.snapshot.as_ref() {
        if previous.generation == generation {
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
    );
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
) -> TrajectorySnapshot {
    let mut reasons = collection.reasons;
    if input.status_truncated {
        reasons.push("status_output_truncated".to_string());
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
        dirty_files: Some(pending.dirty_files),
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
        assert_eq!(Window::Commits(20).kind(), "commits");
        assert_eq!(Window::Commits(20).value(), 20);
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
}
