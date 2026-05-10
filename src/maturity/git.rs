use git2::{BranchType, Repository};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    signal::{MaturitySignal, SignalCollector, SignalType},
    MaturityError, MaturityResult,
};

/// Collector for Git-based maturity signals
pub struct GitSignals;

impl GitSignals {
    pub fn new() -> Self {
        Self
    }

    fn open_repository(&self, path: &Path) -> MaturityResult<Repository> {
        // Try to open the repository directly, or find it by walking up
        Repository::open(path)
            .or_else(|_| Repository::discover(path))
            .map_err(|e| MaturityError::Git(format!("Failed to open repository: {}", e)))
    }

    fn collect_repository_age(&self, repo: &Repository) -> MaturityResult<MaturitySignal> {
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| MaturityError::Git(format!("Failed to create revwalk: {}", e)))?;

        revwalk.push_head().ok();

        let first_commit_time: Option<i64> = revwalk
            .filter_map(|oid| oid.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .map(|commit| commit.time().seconds())
            .min();

        let (value, raw_value, confidence) = match first_commit_time {
            Some(timestamp) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let now_i64: i64 = now as i64;
                let age_days: u64 = ((now_i64 - timestamp).max(0) as u64) / 86400;
                let age_years = age_days as f64 / 365.0;

                // Normalize: 0 years = 0.0, 5+ years = 1.0
                let normalized = (age_years / 5.0).min(1.0);

                (
                    normalized,
                    format!("{:.1} years", age_years),
                    if age_days > 30 { 1.0 } else { 0.8 },
                )
            }
            None => (0.0, "no commits found".to_string(), 0.5),
        };

        Ok(
            MaturitySignal::new(SignalType::Git, "repository_age", value, raw_value)
                .with_confidence(confidence)
                .with_weight(1.5),
        )
    }

    fn collect_commit_frequency(&self, repo: &Repository) -> MaturityResult<MaturitySignal> {
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| MaturityError::Git(format!("Failed to create revwalk: {}", e)))?;

        revwalk.push_head().ok();

        let commits: Vec<_> = revwalk
            .filter_map(|oid| oid.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .map(|commit| commit.time().seconds())
            .collect();

        let (value, raw_value, confidence) = if commits.len() < 2 {
            (0.0, format!("{} commits", commits.len()), 0.5)
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let most_recent = commits.iter().max().copied().unwrap_or(now);
            let oldest = commits.iter().min().copied().unwrap_or(now);
            let time_span_days = ((most_recent - oldest) / 86400).max(1);

            let commits_per_day = commits.len() as f64 / time_span_days as f64;
            let commits_per_week = commits_per_day * 7.0;

            // Normalize: <0.1/week = 0.0, >5/week = 1.0
            let normalized = (commits_per_week / 5.0).min(1.0);

            (
                normalized,
                format!("{:.1} commits/week", commits_per_week),
                if commits.len() > 10 { 1.0 } else { 0.8 },
            )
        };

        Ok(
            MaturitySignal::new(SignalType::Git, "commit_frequency", value, raw_value)
                .with_confidence(confidence)
                .with_weight(1.2),
        )
    }

    fn collect_branch_count(&self, repo: &Repository) -> MaturityResult<MaturitySignal> {
        let branches: Vec<_> = repo
            .branches(Some(BranchType::Local))
            .map_err(|e| MaturityError::Git(format!("Failed to list branches: {}", e)))?
            .filter_map(|b| b.ok())
            .map(|(branch, _)| branch.name().ok().flatten().map(|s| s.to_string()))
            .flatten()
            .collect();

        let branch_count = branches.len();

        // Normalize: 1 branch = 0.0, 10+ branches = 1.0
        let normalized = (branch_count as f64 / 10.0).min(1.0);
        let confidence = if branch_count > 1 { 1.0 } else { 0.7 };

        Ok(MaturitySignal::new(
            SignalType::Git,
            "branch_count",
            normalized,
            format!("{} branches", branch_count),
        )
        .with_confidence(confidence)
        .with_weight(0.8))
    }

    fn collect_protection_status(&self, repo: &Repository) -> MaturityResult<MaturitySignal> {
        // Check for protected branches by looking for main/master with remote tracking
        let branches: Vec<_> = repo
            .branches(Some(BranchType::Local))
            .map_err(|e| MaturityError::Git(format!("Failed to list branches: {}", e)))?
            .filter_map(|b| b.ok())
            .map(|(branch, _)| branch)
            .collect();

        let has_main = branches.iter().any(|b| {
            b.name()
                .ok()
                .flatten()
                .map(|n| n == "main" || n == "master")
                .unwrap_or(false)
        });

        // Check if there's a remote configured (suggests it's hosted somewhere with potential protection)
        let has_remote = repo.remotes().map(|r| !r.is_empty()).unwrap_or(false);

        // Check for GitHub Actions or similar CI files (indirect protection indicator)
        let has_ci = repo
            .workdir()
            .map(|wd| {
                let github_dir = wd.join(".github").join("workflows");
                github_dir.exists()
                    && github_dir
                        .read_dir()
                        .map(|d| d.count() > 0)
                        .unwrap_or(false)
            })
            .unwrap_or(false);

        let score = (has_main as i32) + (has_remote as i32) + (has_ci as i32);
        let normalized = score as f64 / 3.0;
        let confidence = if has_remote { 0.9 } else { 0.6 };

        let indicators = vec![
            if has_main { "main/master branch" } else { "" },
            if has_remote { "remote configured" } else { "" },
            if has_ci { "CI detected" } else { "" },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

        Ok(MaturitySignal::new(
            SignalType::Git,
            "branch_protection",
            normalized,
            if indicators.is_empty() {
                "no protection indicators".to_string()
            } else {
                indicators
            },
        )
        .with_confidence(confidence)
        .with_weight(1.0))
    }

    fn collect_recent_activity(&self, repo: &Repository) -> MaturityResult<MaturitySignal> {
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| MaturityError::Git(format!("Failed to create revwalk: {}", e)))?;

        revwalk.push_head().ok();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Count commits in last 30 days
        let thirty_days_ago = now - (30 * 86400);
        let recent_commits: Vec<_> = revwalk
            .filter_map(|oid| oid.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .filter(|commit| commit.time().seconds() > thirty_days_ago)
            .collect();

        let recent_count = recent_commits.len();

        // Normalize: 0 commits = 0.0, 10+ commits = 1.0
        let normalized = (recent_count as f64 / 10.0).min(1.0);

        let activity_desc = match recent_count {
            0 => "no recent activity",
            1..=2 => "low activity",
            3..=9 => "moderate activity",
            _ => "high activity",
        };

        Ok(MaturitySignal::new(
            SignalType::Git,
            "recent_activity",
            normalized,
            format!("{} commits in 30 days ({})", recent_count, activity_desc),
        )
        .with_confidence(1.0)
        .with_weight(1.3))
    }

    fn collect_committer_diversity(&self, repo: &Repository) -> MaturityResult<MaturitySignal> {
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| MaturityError::Git(format!("Failed to create revwalk: {}", e)))?;

        revwalk.push_head().ok();

        use std::collections::HashSet;
        let committers: HashSet<_> = revwalk
            .filter_map(|oid| oid.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .filter_map(|commit| commit.committer().email().map(|e| e.to_string()))
            .collect();

        let committer_count = committers.len();

        // Normalize: 1 committer = 0.0, 5+ committers = 1.0
        let normalized = ((committer_count as f64 - 1.0) / 4.0).clamp(0.0, 1.0);

        Ok(MaturitySignal::new(
            SignalType::Git,
            "contributor_diversity",
            normalized,
            format!("{} unique committers", committer_count),
        )
        .with_confidence(if committer_count > 0 { 1.0 } else { 0.5 })
        .with_weight(0.9))
    }
}

impl SignalCollector for GitSignals {
    fn signal_type(&self) -> SignalType {
        SignalType::Git
    }

    fn can_collect(&self, path: &Path) -> bool {
        // Check if path is within a git repository
        Repository::open(path).is_ok() || Repository::discover(path).is_ok()
    }

    fn collect(&self, path: &Path) -> MaturityResult<Vec<MaturitySignal>> {
        let repo = self.open_repository(path)?;

        let mut signals = Vec::new();

        match self.collect_repository_age(&repo) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to collect repository age: {}", e),
        }

        match self.collect_commit_frequency(&repo) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to collect commit frequency: {}", e),
        }

        match self.collect_branch_count(&repo) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to collect branch count: {}", e),
        }

        match self.collect_protection_status(&repo) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to collect protection status: {}", e),
        }

        match self.collect_recent_activity(&repo) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to collect recent activity: {}", e),
        }

        match self.collect_committer_diversity(&repo) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to collect committer diversity: {}", e),
        }

        Ok(signals)
    }
}

impl Default for GitSignals {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn create_test_repo() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(base_path)
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(base_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(base_path)
            .output()
            .unwrap();

        temp_dir
    }

    fn add_commit(repo_path: &Path, message: &str) {
        // Create a file
        fs::write(repo_path.join("test.txt"), message).unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo_path)
            .output()
            .unwrap();
    }

    #[test]
    fn test_git_signals_empty_repo() {
        let temp_dir = create_test_repo();
        let collector = GitSignals::new();

        let signals = collector.collect(temp_dir.path()).unwrap();
        assert!(!signals.is_empty());

        // Should have signals but with low values for empty repo
        let age_signal = signals.iter().find(|s| s.name == "repository_age").unwrap();
        assert_eq!(age_signal.value, 0.0);
    }

    #[test]
    fn test_git_signals_with_commits() {
        let temp_dir = create_test_repo();
        add_commit(temp_dir.path(), "Initial commit");

        let collector = GitSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        // Should detect at least one commit
        let freq_signal = signals
            .iter()
            .find(|s| s.name == "commit_frequency")
            .unwrap();
        assert!(freq_signal.raw_value.contains("commits"));
    }

    #[test]
    fn test_git_signals_branch_count() {
        let temp_dir = create_test_repo();
        add_commit(temp_dir.path(), "Initial commit");

        // Create another branch
        Command::new("git")
            .args(["checkout", "-b", "feature-branch"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        let collector = GitSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let branch_signal = signals.iter().find(|s| s.name == "branch_count").unwrap();
        assert!(
            branch_signal.raw_value.contains("2 branches") || branch_signal.raw_value.contains("1")
        );
    }
}
