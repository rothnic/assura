//! Git-backed heat-map signal collection.

use super::{dir_entry, rollup_dirs, HeatBranch, HeatDirectory, HeatTotals};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) struct GitHeat {
    pub(super) available: bool,
    pub(super) branch: HeatBranch,
}

pub(super) fn collect_git_heat(
    project_root: &Path,
    totals: &mut HeatTotals,
    dirs: &mut BTreeMap<String, HeatDirectory>,
) -> GitHeat {
    let Some(repo_root) = git(project_root, &["rev-parse", "--show-toplevel"]) else {
        return GitHeat {
            available: false,
            branch: HeatBranch::default(),
        };
    };
    let scope = GitScope::new(project_root, &repo_root);
    let git_root = scope.repo_root.as_path();

    let mut branch = HeatBranch {
        name: git(git_root, &["rev-parse", "--abbrev-ref", "HEAD"]),
        upstream: git(
            git_root,
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
        ),
        ..HeatBranch::default()
    };
    branch.base = branch.upstream.clone().or_else(|| fallback_base(git_root));
    if let Some(base) = branch.base.as_deref() {
        let branch_range = format!("{base}...HEAD");
        if let Some(counts) = git(
            git_root,
            &["rev-list", "--left-right", "--count", &branch_range],
        ) {
            let parts = counts.split_whitespace().collect::<Vec<_>>();
            if parts.len() == 2 {
                branch.behind = parts[0].parse::<usize>().ok();
                branch.ahead = parts[1].parse::<usize>().ok();
            }
        }
        if let Some(merge_base) = git(git_root, &["merge-base", "HEAD", base]) {
            let commit_range = format!("{merge_base}..HEAD");
            branch.commits_on_branch = git_usize(git_root, &["rev-list", "--count", &commit_range]);
            add_branch_files(git_root, &scope, &merge_base, totals, dirs);
        }
    }

    add_status_files(git_root, &scope, totals, dirs);
    add_numstat(git_root, &scope, totals, dirs);

    GitHeat {
        available: true,
        branch,
    }
}

struct GitScope {
    repo_root: PathBuf,
    pathspec: String,
    prefix: Option<String>,
}

impl GitScope {
    fn new(project_root: &Path, repo_root: &str) -> Self {
        let repo_root = PathBuf::from(repo_root);
        let canonical_project = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        let canonical_repo = repo_root
            .canonicalize()
            .unwrap_or_else(|_| repo_root.clone());
        let pathspec = canonical_project
            .strip_prefix(&canonical_repo)
            .ok()
            .and_then(|path| path.to_str())
            .map(normalize_git_path)
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| ".".to_string());
        let prefix = (pathspec != ".").then(|| format!("{pathspec}/"));
        Self {
            repo_root,
            pathspec,
            prefix,
        }
    }

    fn scoped_path<'a>(&self, path: &'a str) -> Option<&'a str> {
        let path = status_path(path);
        match &self.prefix {
            Some(prefix) => path.strip_prefix(prefix).filter(|path| !path.is_empty()),
            None => Some(path),
        }
    }
}

fn add_status_files(
    git_root: &Path,
    scope: &GitScope,
    totals: &mut HeatTotals,
    dirs: &mut BTreeMap<String, HeatDirectory>,
) {
    let Some(output) = git(
        git_root,
        &["status", "--porcelain=v1", "--", scope.pathspec.as_str()],
    ) else {
        return;
    };
    for line in output.lines() {
        let Some((x, y, path)) = parse_status_line(line) else {
            continue;
        };
        let untracked = x == '?' && y == '?';
        let deleted = x == 'D' || y == 'D';
        let conflicted = x == 'U' || y == 'U' || matches!((x, y), ('A', 'A') | ('D', 'D'));
        let staged = !untracked && x != ' ' && x != '?';
        let unstaged = !untracked && y != ' ';
        let modified = !untracked && !deleted && !conflicted && (staged || unstaged);

        totals.untracked_files += usize::from(untracked);
        totals.deleted_files += usize::from(deleted);
        totals.conflicted_files += usize::from(conflicted);
        totals.staged_files += usize::from(staged);
        totals.unstaged_files += usize::from(unstaged);
        totals.modified_files += usize::from(modified);

        let Some(scoped_path) = scope.scoped_path(path) else {
            continue;
        };
        for dir in rollup_dirs(scoped_path) {
            let entry = dir_entry(dirs, &dir);
            entry.untracked_files += usize::from(untracked);
            entry.deleted_files += usize::from(deleted);
            entry.conflicted_files += usize::from(conflicted);
            entry.staged_files += usize::from(staged);
            entry.unstaged_files += usize::from(unstaged);
            entry.modified_files += usize::from(modified);
        }
    }
}

fn parse_status_line(line: &str) -> Option<(char, char, &str)> {
    let bytes = line.as_bytes();
    if bytes.len() >= 4 && bytes[2] == b' ' {
        return Some((bytes[0] as char, bytes[1] as char, status_path(&line[3..])));
    }
    if bytes.len() >= 3 && bytes[1] == b' ' {
        return Some((bytes[0] as char, ' ', status_path(&line[2..])));
    }
    None
}

fn add_numstat(
    git_root: &Path,
    scope: &GitScope,
    totals: &mut HeatTotals,
    dirs: &mut BTreeMap<String, HeatDirectory>,
) {
    let Some(output) = git(
        git_root,
        &["diff", "HEAD", "--numstat", "--", scope.pathspec.as_str()],
    ) else {
        return;
    };
    for line in output.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 3 {
            continue;
        }
        let additions = parts[0].parse::<usize>().unwrap_or_default();
        let deletions = parts[1].parse::<usize>().unwrap_or_default();
        let Some(scoped_path) = scope.scoped_path(parts[2]) else {
            continue;
        };
        totals.line_additions += additions;
        totals.line_deletions += deletions;
        for dir in rollup_dirs(scoped_path) {
            let entry = dir_entry(dirs, &dir);
            entry.line_additions += additions;
            entry.line_deletions += deletions;
        }
    }
}

fn add_branch_files(
    git_root: &Path,
    scope: &GitScope,
    merge_base: &str,
    totals: &mut HeatTotals,
    dirs: &mut BTreeMap<String, HeatDirectory>,
) {
    let branch_range = format!("{merge_base}..HEAD");
    let Some(output) = git(
        git_root,
        &[
            "diff",
            "--name-only",
            &branch_range,
            "--",
            scope.pathspec.as_str(),
        ],
    ) else {
        return;
    };
    for path in output.lines().filter(|line| !line.trim().is_empty()) {
        let Some(scoped_path) = scope.scoped_path(path) else {
            continue;
        };
        totals.branch_changed_files += 1;
        for dir in rollup_dirs(scoped_path) {
            dir_entry(dirs, &dir).branch_changed_files += 1;
        }
    }
}

fn status_path(path: &str) -> &str {
    path.rsplit_once(" -> ")
        .map(|(_, path)| path)
        .unwrap_or(path)
}

fn fallback_base(project_root: &Path) -> Option<String> {
    ["origin/main", "origin/master", "main", "master"]
        .iter()
        .find(|candidate| {
            git(
                project_root,
                &["rev-parse", "--verify", "--quiet", candidate],
            )
            .is_some()
        })
        .map(|candidate| (*candidate).to_string())
}

fn git_usize(project_root: &Path, args: &[&str]) -> Option<usize> {
    git(project_root, args)?.trim().parse::<usize>().ok()
}

fn git(project_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(project_root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

fn normalize_git_path(path: &str) -> String {
    path.replace('\\', "/").trim_matches('/').to_string()
}
