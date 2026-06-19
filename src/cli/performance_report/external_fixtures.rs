//! Opt-in pinned external Git fixtures for performance reports.

use super::external_fixture_catalog::external_fixture_spec;
use super::external_fixture_catalog::ExternalFixtureSpec;
use super::fixture_io::{write_file, write_lslint_compatible_configs};
use super::fixtures::FixtureKind;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn materialize_external_fixture(kind: FixtureKind, root: &Path) -> Result<(), String> {
    let spec = external_fixture_spec(kind).ok_or("fixture kind is not external")?;
    let cache_root = external_fixture_cache_root()?;
    materialize_external_fixture_spec(spec, root, &cache_root)
}

fn materialize_external_fixture_spec(
    spec: ExternalFixtureSpec,
    root: &Path,
    cache_root: &Path,
) -> Result<(), String> {
    fs::create_dir_all(cache_root)
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
    } else if git_output(
        ["rev-parse", &format!("{}^{{commit}}", spec.revision)],
        Some(&cache_dir),
    )
    .is_err()
    {
        run_git(["fetch", "--tags", "--quiet", "origin"], Some(&cache_dir))?;
    }

    let resolved = git_output(
        ["rev-parse", &format!("{}^{{commit}}", spec.revision)],
        Some(&cache_dir),
    )?;
    run_git(["checkout", "--quiet", &resolved], Some(&cache_dir))?;
    copy_without_git(&cache_dir, root)?;
    write_lslint_compatible_configs(root, spec.ls_lint_config)?;
    write_file(
        root.join(".assura/source-revision.txt"),
        &format!("{resolved}\n"),
    )
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
    let resolved_target = if target.is_absolute() {
        target.clone()
    } else {
        source
            .parent()
            .map(|parent| parent.join(&target))
            .unwrap_or_else(|| target.clone())
    };
    if !resolved_target.exists() {
        return Ok(());
    }

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

#[cfg(test)]
mod tests {
    use super::{copy_symlink, materialize_external_fixture_spec};
    use crate::cli::performance_report::external_fixture_catalog::ExternalFixtureSpec;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn materialize_external_fixture_uses_pinned_revision_and_reuses_cache() {
        let upstream = TempDir::new().unwrap();
        run_git(["init", "--quiet", upstream.path().to_str().unwrap()], None);
        fs::write(upstream.path().join("README.md"), "# Fixture\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("README.md", upstream.path().join("README-link.md")).unwrap();
        run_git(["-C", upstream.path().to_str().unwrap(), "add", "."], None);
        run_git(
            [
                "-C",
                upstream.path().to_str().unwrap(),
                "-c",
                "user.email=test@example.com",
                "-c",
                "user.name=Test",
                "commit",
                "--quiet",
                "-m",
                "initial",
            ],
            None,
        );
        let revision = git_output(["-C", upstream.path().to_str().unwrap(), "rev-parse", "HEAD"]);
        fs::write(upstream.path().join("README.md"), "# Changed\n").unwrap();

        let repository: &'static str = Box::leak(
            upstream
                .path()
                .to_string_lossy()
                .into_owned()
                .into_boxed_str(),
        );
        let revision: &'static str = Box::leak(revision.into_boxed_str());
        let spec = ExternalFixtureSpec {
            fixture_id: "local_external",
            repository,
            revision,
            ls_lint_config: r#"
ignore:
  - .assura/**
  - .git/**
ls:
  .md: regex:^README$
"#,
        };
        let cache = TempDir::new().unwrap();
        let first_destination = TempDir::new().unwrap();

        materialize_external_fixture_spec(spec, first_destination.path(), cache.path()).unwrap();

        assert_eq!(
            read_text_with_normalized_newlines(first_destination.path().join("README.md")),
            "# Fixture\n"
        );
        assert!(first_destination.path().join(".ls-lint.yml").exists());
        assert!(first_destination.path().join(".assura/config.yml").exists());
        assert_eq!(
            fs::read_to_string(first_destination.path().join(".assura/source-revision.txt"))
                .unwrap()
                .trim(),
            spec.revision
        );
        assert!(!first_destination.path().join(".git").exists());
        #[cfg(unix)]
        {
            let link_metadata =
                fs::symlink_metadata(first_destination.path().join("README-link.md")).unwrap();
            assert!(link_metadata.file_type().is_symlink());
            assert_eq!(
                fs::read_link(first_destination.path().join("README-link.md")).unwrap(),
                std::path::PathBuf::from("README.md")
            );
        }

        fs::remove_dir_all(upstream.path()).unwrap();
        let second_destination = TempDir::new().unwrap();
        materialize_external_fixture_spec(spec, second_destination.path(), cache.path()).unwrap();

        assert_eq!(
            read_text_with_normalized_newlines(second_destination.path().join("README.md")),
            "# Fixture\n"
        );
        assert!(fs::read_dir(cache.path()).unwrap().next().is_some());
    }

    fn read_text_with_normalized_newlines(path: impl AsRef<std::path::Path>) -> String {
        fs::read_to_string(path).unwrap().replace("\r\n", "\n")
    }

    fn run_git<const N: usize>(args: [&str; N], current_dir: Option<&std::path::Path>) {
        let mut command = Command::new("git");
        command.args(args);
        if let Some(current_dir) = current_dir {
            command.current_dir(current_dir);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "git failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_output<const N: usize>(args: [&str; N]) -> String {
        let output = Command::new("git").args(args).output().unwrap();
        assert!(
            output.status.success(),
            "git failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[cfg(unix)]
    #[test]
    fn copy_symlink_skips_broken_links() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("broken-link");
        let destination = temp.path().join("copied-link");

        std::os::unix::fs::symlink(temp.path().join("missing-target"), &source).unwrap();

        copy_symlink(&source, &destination).unwrap();

        assert!(!destination.exists());
        assert!(std::fs::symlink_metadata(&destination).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn copy_symlink_preserves_valid_links() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.txt");
        let source = temp.path().join("valid-link");
        let destination = temp.path().join("copied-link");

        std::fs::write(&target, "ok").unwrap();
        std::os::unix::fs::symlink("target.txt", &source).unwrap();

        copy_symlink(&source, &destination).unwrap();

        assert!(std::fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            std::fs::read_link(&destination).unwrap(),
            Path::new("target.txt")
        );
    }

    #[cfg(unix)]
    use std::path::Path;
}
