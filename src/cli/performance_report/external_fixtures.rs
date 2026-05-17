//! Opt-in pinned external Git fixtures for performance reports.

use super::fixture_io::{write_configs, write_file};
use super::fixtures::FixtureKind;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn materialize_external_fixture(kind: FixtureKind, root: &Path) -> Result<(), String> {
    let spec = external_fixture_spec(kind).ok_or("fixture kind is not external")?;
    let cache_root = external_fixture_cache_root()?;
    fs::create_dir_all(&cache_root)
        .map_err(|error| format!("create cache {}: {error}", cache_root.display()))?;
    let cache_dir = cache_root.join(cache_key(spec.repository, spec.revision));

    if !cache_dir.exists() {
        run_git(
            [
                "clone",
                "--quiet",
                "--no-checkout",
                "--filter=blob:none",
                spec.repository,
                cache_dir.to_str().ok_or("cache path is not valid UTF-8")?,
            ],
            None,
        )?;
    } else {
        run_git(["fetch", "--tags", "--quiet", "origin"], Some(&cache_dir))?;
    }

    let resolved = git_output(
        ["rev-parse", &format!("{}^{{commit}}", spec.revision)],
        Some(&cache_dir),
    )?;
    run_git(["checkout", "--quiet", &resolved], Some(&cache_dir))?;
    copy_without_git(&cache_dir, root)?;
    write_configs(root, spec.assura_config, spec.ls_lint_config)?;
    write_file(
        root.join(".assura/source-revision.txt"),
        &format!("{resolved}\n"),
    )
}

struct ExternalFixtureSpec {
    repository: &'static str,
    revision: &'static str,
    assura_config: &'static str,
    ls_lint_config: &'static str,
}

fn external_fixture_spec(kind: FixtureKind) -> Option<ExternalFixtureSpec> {
    match kind {
        FixtureKind::PinnedNextJs => Some(ExternalFixtureSpec {
            repository: "https://github.com/vercel/next.js",
            revision: "v15.0.0",
            assura_config: EXTERNAL_FRONTEND_ASSURA_CONFIG,
            ls_lint_config: EXTERNAL_FRONTEND_LS_LINT_CONFIG,
        }),
        FixtureKind::PinnedMdBook => Some(ExternalFixtureSpec {
            repository: "https://github.com/rust-lang/mdBook",
            revision: "v0.4.48",
            assura_config: EXTERNAL_RUST_ASSURA_CONFIG,
            ls_lint_config: EXTERNAL_RUST_LS_LINT_CONFIG,
        }),
        _ => None,
    }
}

fn external_fixture_cache_root() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("ASSURA_PERF_EXTERNAL_FIXTURE_CACHE") {
        return Ok(PathBuf::from(path));
    }
    let cwd = std::env::current_dir().map_err(|error| format!("read current dir: {error}"))?;
    Ok(cwd.join("target/performance/external-fixture-cache"))
}

fn run_git<const N: usize>(args: [&str; N], current_dir: Option<&Path>) -> Result<(), String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git exited {:?}; stdout: {}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn git_output<const N: usize>(
    args: [&str; N],
    current_dir: Option<&Path>,
) -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "git exited {:?}; stdout: {}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn cache_key(repository: &str, revision: &str) -> String {
    format!("{repository}-{revision}")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn copy_without_git(source: &Path, destination: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source)
        .map_err(|error| format!("read source {}: {error}", source.display()))?
    {
        let entry = entry.map_err(|error| format!("read source entry: {error}"))?;
        if entry.file_name() == ".git" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = fs::symlink_metadata(&source_path)
            .map_err(|error| format!("metadata {}: {error}", source_path.display()))?
            .file_type();
        if file_type.is_symlink() {
            copy_symlink(&source_path, &destination_path)?;
        } else if file_type.is_dir() {
            fs::create_dir_all(&destination_path).map_err(|error| {
                format!("create destination {}: {error}", destination_path.display())
            })?;
            copy_without_git(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!(
                    "copy {} to {}: {error}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn copy_symlink(source: &Path, destination: &Path) -> Result<(), String> {
    let target = fs::read_link(source)
        .map_err(|error| format!("read link {}: {error}", source.display()))?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, destination).map_err(|error| {
            format!(
                "symlink {} -> {}: {error}",
                destination.display(),
                target.display()
            )
        })?;
    }

    #[cfg(windows)]
    {
        let target_path = source
            .parent()
            .map(|parent| parent.join(&target))
            .unwrap_or_else(|| target.clone());
        let metadata = fs::metadata(&target_path).map_err(|error| {
            format!(
                "metadata symlink target {} for {}: {error}",
                target_path.display(),
                source.display()
            )
        })?;
        if metadata.is_dir() {
            std::os::windows::fs::symlink_dir(&target, destination)
        } else {
            std::os::windows::fs::symlink_file(&target, destination)
        }
        .map_err(|error| {
            format!(
                "symlink {} -> {}: {error}",
                destination.display(),
                target.display()
            )
        })?;
    }

    Ok(())
}

const EXTERNAL_FRONTEND_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - node_modules/**
  - packages/*/node_modules/**
  - examples/*/node_modules/**
  - test/**/node_modules/**
  - .next/**
  - packages/*/.next/**
  - examples/*/.next/**
  - dist/**
  - packages/*/dist/**
  - coverage/**
  - .turbo/**
  - .vercel/**
ls:
  .dir: regex:^[A-Za-z0-9._-]+$
  .*: regex:^[A-Za-z0-9._-]+$
"#;

const EXTERNAL_FRONTEND_ASSURA_CONFIG: &str = r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.*": regex:^[A-Za-z0-9._-]+$
    directories:
      naming: regex:^[A-Za-z0-9._-]+$
exclude:
  - ".assura/**"
  - ".git/**"
  - "node_modules/**"
  - "packages/*/node_modules/**"
  - "examples/*/node_modules/**"
  - "test/**/node_modules/**"
  - ".next/**"
  - "packages/*/.next/**"
  - "examples/*/.next/**"
  - "dist/**"
  - "packages/*/dist/**"
  - "coverage/**"
  - ".turbo/**"
  - ".vercel/**"
"#;

const EXTERNAL_RUST_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - target/**
ls:
  .dir: regex:^[A-Za-z0-9._-]+$
  .*: regex:^[A-Za-z0-9._-]+$
  .rs: snake_case
  .md: regex:^[A-Za-z0-9._-]+$
  .toml: regex:^[A-Za-z0-9._-]+$
"#;

const EXTERNAL_RUST_ASSURA_CONFIG: &str = r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.*": regex:^[A-Za-z0-9._-]+$
        "*.rs": snake_case
        "*.md": regex:^[A-Za-z0-9._-]+$
        "*.toml": regex:^[A-Za-z0-9._-]+$
    directories:
      naming: regex:^[A-Za-z0-9._-]+$
exclude:
  - ".assura/**"
  - ".git/**"
  - "target/**"
"#;
