//! Cached LS-Lint setup for performance comparisons.

use super::ToolAvailability;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct PreparedLsLint {
    pub(super) status: ToolAvailability,
    pub(super) binary_path: Option<PathBuf>,
    pub(super) execution_mode: Option<&'static str>,
    install_dir: Option<PathBuf>,
}

impl Drop for PreparedLsLint {
    fn drop(&mut self) {
        if let Some(install_dir) = &self.install_dir {
            let _ = fs::remove_dir_all(install_dir);
        }
    }
}

pub(super) fn prepare_ls_lint(ls_lint_package: &str) -> PreparedLsLint {
    let install_dir = std::env::temp_dir().join(format!(
        "assura_lslint_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ));

    if let Err(error) = fs::create_dir_all(&install_dir) {
        return unavailable(
            format!(
                "failed to create LS-Lint install dir {}: {error}",
                install_dir.display()
            ),
            None,
        );
    }

    match npm_install_lslint_command(&install_dir, ls_lint_package).output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            return unavailable(
                format!(
                    "npm install exited {:?}; stdout: {}; stderr: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
                Some(install_dir),
            );
        }
        Err(error) => {
            return unavailable(
                format!("failed to run npm install: {error}"),
                Some(install_dir),
            );
        }
    }

    let binary_path = match ls_lint_binary_path(&install_dir) {
        Some(binary_path) => binary_path,
        None => {
            return unavailable(
                format!(
                    "no native LS-Lint binary is packaged for {} {}",
                    std::env::consts::OS,
                    std::env::consts::ARCH
                ),
                Some(install_dir),
            );
        }
    };
    if !binary_path.exists() {
        return unavailable(
            format!(
                "native LS-Lint binary was not installed at {}",
                binary_path.display()
            ),
            Some(install_dir),
        );
    }

    match Command::new(&binary_path).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            PreparedLsLint {
                status: ToolAvailability {
                    available: true,
                    version: Some(if stdout.is_empty() {
                        ls_lint_package.to_string()
                    } else {
                        stdout
                    }),
                    blocker: None,
                },
                binary_path: Some(binary_path),
                execution_mode: Some("native-binary-from-pinned-npm-package"),
                install_dir: Some(install_dir),
            }
        }
        Ok(output) => unavailable(
            format!(
                "cached LS-Lint exited {:?}; stdout: {}; stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Some(install_dir),
        ),
        Err(error) => unavailable(
            format!("failed to run cached LS-Lint: {error}"),
            Some(install_dir),
        ),
    }
}

fn unavailable(blocker: String, install_dir: Option<PathBuf>) -> PreparedLsLint {
    PreparedLsLint {
        status: ToolAvailability {
            available: false,
            version: None,
            blocker: Some(blocker),
        },
        binary_path: None,
        execution_mode: None,
        install_dir,
    }
}

fn npm_install_lslint_command(install_dir: &Path, ls_lint_package: &str) -> Command {
    let mut command = Command::new("npm");
    command
        .env("NPM_CONFIG_FETCH_RETRIES", "0")
        .env("NPM_CONFIG_FETCH_TIMEOUT", "15000")
        .env("NPM_CONFIG_CACHE", install_dir.join(".npm-cache"))
        .args(["install", "--no-audit", "--no-fund", "--prefix"])
        .arg(install_dir)
        .arg(ls_lint_package);
    command
}

fn ls_lint_binary_path(install_dir: &Path) -> Option<PathBuf> {
    Some(
        install_dir
            .join("node_modules")
            .join("@ls-lint")
            .join("ls-lint")
            .join("bin")
            .join(native_ls_lint_binary_name(
                std::env::consts::OS,
                std::env::consts::ARCH,
            )?),
    )
}

fn native_ls_lint_binary_name(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("macos", "x86_64") => Some("ls-lint-darwin-amd64"),
        ("macos", "aarch64") => Some("ls-lint-darwin-arm64"),
        ("linux", "x86_64") => Some("ls-lint-linux-amd64"),
        ("linux", "aarch64") => Some("ls-lint-linux-arm64"),
        ("linux", "s390x") => Some("ls-lint-linux-s390x"),
        ("linux", "powerpc64") => Some("ls-lint-linux-ppc64le"),
        ("windows", "x86_64") => Some("ls-lint-windows-amd64.exe"),
        _ => None,
    }
}

#[cfg(test)]
fn ls_lint_node_wrapper_path(install_dir: &Path) -> PathBuf {
    install_dir
        .join("node_modules")
        .join(".bin")
        .join(if cfg!(windows) {
            "ls-lint.cmd"
        } else {
            "ls-lint"
        })
}

#[cfg(test)]
mod tests {
    use super::{ls_lint_binary_path, ls_lint_node_wrapper_path, native_ls_lint_binary_name};
    use std::path::Path;

    #[test]
    fn native_binary_name_matches_packaged_ls_lint_targets() {
        assert_eq!(
            native_ls_lint_binary_name("macos", "x86_64"),
            Some("ls-lint-darwin-amd64")
        );
        assert_eq!(
            native_ls_lint_binary_name("macos", "aarch64"),
            Some("ls-lint-darwin-arm64")
        );
        assert_eq!(
            native_ls_lint_binary_name("linux", "x86_64"),
            Some("ls-lint-linux-amd64")
        );
        assert_eq!(
            native_ls_lint_binary_name("linux", "aarch64"),
            Some("ls-lint-linux-arm64")
        );
        assert_eq!(
            native_ls_lint_binary_name("linux", "s390x"),
            Some("ls-lint-linux-s390x")
        );
        assert_eq!(
            native_ls_lint_binary_name("linux", "powerpc64"),
            Some("ls-lint-linux-ppc64le")
        );
        assert_eq!(
            native_ls_lint_binary_name("windows", "x86_64"),
            Some("ls-lint-windows-amd64.exe")
        );
        assert_eq!(native_ls_lint_binary_name("freebsd", "x86_64"), None);
    }

    #[test]
    fn prepared_path_targets_native_binary_not_node_wrapper() {
        let install_dir = Path::new("/tmp/assura_lslint_test");
        let binary_path = ls_lint_binary_path(install_dir).unwrap();
        let wrapper_path = ls_lint_node_wrapper_path(install_dir);

        assert!(binary_path.ends_with(
            native_ls_lint_binary_name(std::env::consts::OS, std::env::consts::ARCH)
                .unwrap_or("unsupported")
        ));
        let components: Vec<_> = binary_path
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect();
        assert!(components.windows(4).any(|window| {
            window.iter().map(|component| component.as_ref()).eq([
                "node_modules",
                "@ls-lint",
                "ls-lint",
                "bin",
            ])
        }));
        assert_ne!(binary_path, wrapper_path);
    }
}
