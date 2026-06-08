//! Quality gate planning command.

use crate::cli::args::{QualityPhase, QualityPlanFormat};
use crate::cli::{ConfigDiscovery, ExitCode};
use crate::config::config::{Config, QualityScopeConfig};
use crate::config::loader::ConfigLoader;
use glob::Pattern;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Options passed from the CLI into quality gate planning.
pub struct QualityPlanCommandOptions {
    /// Project root path used for config discovery and git diffs.
    pub path: Option<PathBuf>,
    /// Explicit Assura configuration path.
    pub config: Option<PathBuf>,
    /// File containing changed paths, or `-` for stdin.
    pub files_from: Option<String>,
    /// Base git revision for diff-based planning.
    pub base: Option<String>,
    /// Head git revision for diff-based planning.
    pub head: Option<String>,
    /// Workflow phase to plan.
    pub phase: QualityPhase,
    /// Output format.
    pub format: QualityPlanFormat,
}

/// Machine-readable quality gate plan.
#[derive(Debug, Serialize)]
pub struct QualityPlan {
    /// Schema version for consumers.
    pub schema_version: &'static str,
    /// Project root used for the plan.
    pub project_root: PathBuf,
    /// Config path used for the plan.
    pub config_path: PathBuf,
    /// Workflow phase used for cumulative checks.
    pub phase: String,
    /// Changed paths considered by the planner.
    pub changed_paths: Vec<String>,
    /// Matched quality scopes.
    pub scopes: Vec<QualityScopeMatch>,
    /// De-duplicated checks required by all matched scopes.
    pub checks: Vec<String>,
}

/// A matched quality scope.
#[derive(Debug, Serialize)]
pub struct QualityScopeMatch {
    /// Scope id from `quality.scopes`.
    pub id: String,
    /// Changed paths that matched this scope.
    pub matched_paths: Vec<String>,
    /// Checks required by this scope for the requested phase.
    pub checks: Vec<String>,
}

/// Plan quality gates for changed files.
pub async fn quality_plan_command(options: QualityPlanCommandOptions) -> ExitCode {
    match build_quality_plan(&options) {
        Ok(plan) => {
            println!("{}", render_quality_plan(&plan, options.format));
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::ConfigurationError
        }
    }
}

fn build_quality_plan(options: &QualityPlanCommandOptions) -> Result<QualityPlan, String> {
    let path = options
        .path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let config_path = match &options.config {
        Some(path) => path.clone(),
        None => ConfigDiscovery::find_config_path(&path)
            .filter(|path| path.exists())
            .ok_or_else(|| format!("no .assura/config.yml found for {:?}", path))?,
    };
    let project_root = config_path
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.clone());
    let config = ConfigLoader::load(&config_path).map_err(|error| error.to_string())?;
    let changed_paths = collect_changed_paths(options, &project_root)?;
    plan_from_config(
        config,
        project_root,
        config_path,
        options.phase,
        changed_paths,
    )
}

fn plan_from_config(
    config: Config,
    project_root: PathBuf,
    config_path: PathBuf,
    phase: QualityPhase,
    changed_paths: Vec<String>,
) -> Result<QualityPlan, String> {
    let quality = config
        .quality
        .ok_or_else(|| "quality.scopes is not configured".to_string())?;
    let phase_key = phase.as_config_key();
    let mut scopes = Vec::new();
    let mut checks = Vec::new();

    for (scope_id, scope) in sorted_scopes(&quality.scopes) {
        let matched_paths = changed_paths
            .iter()
            .filter(|path| scope_matches_path(scope, path))
            .cloned()
            .collect::<Vec<_>>();
        if matched_paths.is_empty() {
            continue;
        }

        let scope_checks = scope.checks_for_phase(phase_key);
        append_unique(&mut checks, &scope_checks);
        scopes.push(QualityScopeMatch {
            id: scope_id.to_string(),
            matched_paths,
            checks: scope_checks,
        });
    }

    Ok(QualityPlan {
        schema_version: "assura.quality-plan.v1",
        project_root,
        config_path,
        phase: phase_key.to_string(),
        changed_paths,
        scopes,
        checks,
    })
}

fn sorted_scopes(
    scopes: &std::collections::HashMap<String, QualityScopeConfig>,
) -> BTreeMap<&str, &QualityScopeConfig> {
    scopes
        .iter()
        .map(|(id, scope)| (id.as_str(), scope))
        .collect::<BTreeMap<_, _>>()
}

fn collect_changed_paths(
    options: &QualityPlanCommandOptions,
    project_root: &Path,
) -> Result<Vec<String>, String> {
    let paths = if let Some(files_from) = &options.files_from {
        read_changed_paths(files_from)?
    } else if let (Some(base), Some(head)) = (&options.base, &options.head) {
        git_changed_paths(project_root, base, head)?
    } else {
        git_changed_paths(project_root, "HEAD", "--")?
    };

    Ok(paths
        .into_iter()
        .map(|path| normalize_changed_path(&path))
        .filter(|path| !path.is_empty())
        .collect())
}

fn read_changed_paths(files_from: &str) -> Result<Vec<String>, String> {
    let content = if files_from == "-" {
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .map_err(|error| format!("failed to read stdin: {error}"))?;
        content
    } else {
        std::fs::read_to_string(files_from)
            .map_err(|error| format!("failed to read {files_from}: {error}"))?
    };

    Ok(content.lines().map(str::to_string).collect())
}

fn git_changed_paths(project_root: &Path, base: &str, head: &str) -> Result<Vec<String>, String> {
    if head == "--" {
        let mut paths = git_output_lines(project_root, ["diff", "--name-only", base])?;
        let untracked =
            git_output_lines(project_root, ["ls-files", "--others", "--exclude-standard"])?;
        append_unique(&mut paths, &untracked);
        return Ok(paths);
    }

    git_output_lines(
        project_root,
        ["diff", "--name-only", &format!("{base}...{head}")],
    )
}

fn git_output_lines<const N: usize>(
    project_root: &Path,
    args: [&str; N],
) -> Result<Vec<String>, String> {
    let mut command = Command::new("git");
    command.current_dir(project_root);
    command.args(args);
    let output = command
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

fn scope_matches_path(scope: &QualityScopeConfig, path: &str) -> bool {
    scope
        .paths
        .iter()
        .any(|pattern| path_matches(pattern, path))
}

fn path_matches(pattern: &str, path: &str) -> bool {
    let pattern = normalize_changed_path(pattern);
    let path = normalize_changed_path(path);
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
        return Pattern::new(&pattern)
            .map(|pattern| pattern.matches_path(Path::new(&path)))
            .unwrap_or(false);
    }
    path == pattern
}

fn normalize_changed_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn render_quality_plan(plan: &QualityPlan, format: QualityPlanFormat) -> String {
    match format {
        QualityPlanFormat::Text => render_text(plan),
        QualityPlanFormat::Json => serde_json::to_string_pretty(plan).unwrap_or_default(),
        QualityPlanFormat::Github => render_github(plan),
    }
}

fn render_text(plan: &QualityPlan) -> String {
    let mut output = String::new();
    output.push_str("Assura quality plan\n");
    output.push_str("===================\n");
    output.push_str(&format!("Phase: {}\n", plan.phase));
    output.push_str(&format!("Changed paths: {}\n", plan.changed_paths.len()));
    output.push_str(&format!("Matched scopes: {}\n", plan.scopes.len()));
    output.push_str("Checks:\n");
    if plan.checks.is_empty() {
        output.push_str("  (none)\n");
    } else {
        for check in &plan.checks {
            output.push_str(&format!("  - {check}\n"));
        }
    }
    output
}

fn render_github(plan: &QualityPlan) -> String {
    let mut output = String::new();
    for scope in &plan.scopes {
        output.push_str(&format!("scope_{}=true\n", sanitize_output_key(&scope.id)));
    }
    output.push_str(&format!("checks={}\n", plan.checks.join(",")));
    output.push_str(&format!("changed_count={}\n", plan.changed_paths.len()));
    output.push_str(&format!("scope_count={}\n", plan.scopes.len()));
    output
}

fn sanitize_output_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn append_unique(target: &mut Vec<String>, source: &[String]) {
    for value in source {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::QualityConfig;
    use std::collections::HashMap;

    #[test]
    fn path_patterns_match_exact_glob_and_tree_prefixes() {
        assert!(path_matches("Cargo.toml", "Cargo.toml"));
        assert!(path_matches("src/**", "src/main.rs"));
        assert!(path_matches("src/**", "src"));
        assert!(path_matches(
            "website/public/install.*",
            "website/public/install.sh"
        ));
        assert!(!path_matches("docs/**", "src/lib.rs"));
    }

    #[test]
    fn plan_uses_cumulative_phase_checks() {
        let mut scopes = HashMap::new();
        scopes.insert(
            "rust".to_string(),
            QualityScopeConfig {
                paths: vec!["src/**".to_string()],
                always: vec!["assura:self-check".to_string()],
                frequent: vec!["verify:check".to_string()],
                pre_push: vec!["verify:test".to_string()],
                pr: vec!["verify:pr".to_string()],
                merge: vec!["coverage".to_string()],
                release: Vec::new(),
                scheduled: Vec::new(),
            },
        );
        let config = Config::new().with_quality(QualityConfig { scopes });

        let plan = plan_from_config(
            config,
            PathBuf::from("."),
            PathBuf::from(".assura/config.yml"),
            QualityPhase::Merge,
            vec!["src/main.rs".to_string()],
        )
        .unwrap();

        assert_eq!(
            plan.checks,
            vec![
                "assura:self-check",
                "verify:check",
                "verify:test",
                "verify:pr",
                "coverage"
            ]
        );
    }

    #[test]
    fn default_git_changed_paths_include_untracked_files() {
        let project = tempfile::tempdir().unwrap();
        run_git(project.path(), ["init"]);
        run_git(
            project.path(),
            ["config", "user.email", "assura@example.test"],
        );
        run_git(project.path(), ["config", "user.name", "Assura Test"]);
        std::fs::write(project.path().join("tracked.rs"), "fn main() {}\n").unwrap();
        run_git(project.path(), ["add", "tracked.rs"]);
        run_git(project.path(), ["commit", "-m", "initial"]);

        std::fs::write(project.path().join("new_quality.rs"), "fn quality() {}\n").unwrap();

        let paths = git_changed_paths(project.path(), "HEAD", "--").unwrap();
        assert!(paths.contains(&"new_quality.rs".to_string()), "{paths:?}");
    }

    fn run_git<const N: usize>(path: &Path, args: [&str; N]) {
        let output = Command::new("git")
            .current_dir(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
