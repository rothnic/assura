//! Git hook installation and status management for Assura.
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HookError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git directory not found")]
    GitNotFound,

    #[error("Hook already exists: {0}")]
    AlreadyExists(String),

    #[error("Hook not found: {0}")]
    NotFound(String),

    #[error("Invalid hook type: {0}")]
    InvalidType(String),
}

pub type HookResult<T> = Result<T, HookError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    PreCommit,
    PrePush,
    PostCheckout,
}

impl HookType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PrePush => "pre-push",
            HookType::PostCheckout => "post-checkout",
        }
    }

    pub fn all() -> Vec<HookType> {
        vec![
            HookType::PreCommit,
            HookType::PrePush,
            HookType::PostCheckout,
        ]
    }
}

impl std::str::FromStr for HookType {
    type Err = HookError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pre-commit" => Ok(HookType::PreCommit),
            "pre-push" => Ok(HookType::PrePush),
            "post-checkout" => Ok(HookType::PostCheckout),
            _ => Err(HookError::InvalidType(s.to_string())),
        }
    }
}

pub struct GitHooksManager {
    git_hooks_dir: PathBuf,
    assura_hooks_dir: PathBuf,
}

impl GitHooksManager {
    pub fn new(project_root: impl AsRef<Path>) -> HookResult<Self> {
        let project_root = project_root.as_ref();
        let git_hooks_dir = resolve_git_hooks_dir(project_root)?;
        let assura_hooks_dir = project_root.join(".assura").join("hooks");

        Ok(Self {
            git_hooks_dir,
            assura_hooks_dir,
        })
    }

    pub fn install_all(&self, force: bool) -> HookResult<Vec<HookType>> {
        std::fs::create_dir_all(&self.assura_hooks_dir)?;

        let mut installed = Vec::new();

        for hook_type in HookType::all() {
            match self.install(hook_type, force) {
                Ok(_) => installed.push(hook_type),
                Err(HookError::AlreadyExists(_)) if !force => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(installed)
    }

    pub fn install(&self, hook_type: HookType, force: bool) -> HookResult<()> {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);

        // Check if already exists
        if git_hook_path.exists() && !force {
            return Err(HookError::AlreadyExists(hook_name.to_string()));
        }

        // Create the assura hook script
        let hook_content = self.generate_hook_content(hook_type)?;
        std::fs::write(&assura_hook_path, hook_content)?;

        // Make it executable (on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&assura_hook_path)?.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&assura_hook_path, permissions)?;
        }

        // Create git hook that delegates to assura
        let git_hook_content = format!(
            r#"#!/bin/sh
# Git hook managed by Assura
# This file was auto-generated. Do not modify manually.

ASSURA_HOOK="{}"

if [ -f "$ASSURA_HOOK" ]; then
    exec "$ASSURA_HOOK" "$@"
else
    echo "Warning: Assura hook not found at $ASSURA_HOOK" >&2
    exit 0
fi
"#,
            assura_hook_path.display()
        );

        std::fs::write(&git_hook_path, git_hook_content)?;

        // Make git hook executable (on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&git_hook_path)?.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&git_hook_path, permissions)?;
        }

        Ok(())
    }

    pub fn uninstall(&self, hook_type: HookType) -> HookResult<()> {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);

        // Check if it's an assura-managed hook
        if git_hook_path.exists() {
            let content = std::fs::read_to_string(&git_hook_path)?;
            if content.contains("Git hook managed by Assura") {
                std::fs::remove_file(&git_hook_path)?;
            }
        }

        if assura_hook_path.exists() {
            std::fs::remove_file(&assura_hook_path)?;
        }

        Ok(())
    }

    pub fn uninstall_all(&self) -> HookResult<Vec<HookType>> {
        let mut uninstalled = Vec::new();

        for hook_type in HookType::all() {
            if self.status(hook_type).is_installed {
                self.uninstall(hook_type)?;
                uninstalled.push(hook_type);
            }
        }

        Ok(uninstalled)
    }

    pub fn status(&self, hook_type: HookType) -> HookStatus {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);

        let is_installed = git_hook_path.exists() || assura_hook_path.exists();
        let is_managed = if git_hook_path.exists() {
            std::fs::read_to_string(&git_hook_path)
                .map(|content| content.contains("Git hook managed by Assura"))
                .unwrap_or(false)
        } else {
            false
        };

        HookStatus {
            hook_type,
            is_installed,
            is_managed,
            git_runnable: is_runnable(&git_hook_path),
            assura_runnable: is_runnable(&assura_hook_path),
            git_path: git_hook_path,
            assura_path: assura_hook_path,
        }
    }

    pub fn all_status(&self) -> Vec<HookStatus> {
        HookType::all()
            .into_iter()
            .map(|t| self.status(t))
            .collect()
    }

    fn generate_hook_content(&self, hook_type: HookType) -> HookResult<String> {
        let content = match hook_type {
            HookType::PreCommit => include_str!("../../.assura/hooks/pre-commit"),
            HookType::PrePush => include_str!("../../.assura/hooks/pre-push"),
            HookType::PostCheckout => include_str!("../../.assura/hooks/post-checkout"),
        };

        Ok(content.to_string())
    }
}

fn resolve_git_hooks_dir(project_root: &Path) -> HookResult<PathBuf> {
    let git_path = project_root.join(".git");
    if git_path.is_dir() {
        return Ok(git_path.join("hooks"));
    }
    if git_path.is_file() {
        let git_dir = resolve_gitdir_file(&git_path)?;
        return Ok(resolve_common_git_dir(&git_dir).join("hooks"));
    }
    Err(HookError::GitNotFound)
}

fn resolve_gitdir_file(git_path: &Path) -> HookResult<PathBuf> {
    let content = std::fs::read_to_string(git_path)?;
    let Some(first_line) = content.lines().next() else {
        return Err(HookError::GitNotFound);
    };
    let Some(raw_git_dir) = first_line.trim().strip_prefix("gitdir:") else {
        return Err(HookError::GitNotFound);
    };
    let git_dir = PathBuf::from(raw_git_dir.trim());
    if git_dir.is_absolute() {
        Ok(git_dir)
    } else {
        Ok(git_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(git_dir))
    }
}

fn resolve_common_git_dir(git_dir: &Path) -> PathBuf {
    let common_dir_file = git_dir.join("commondir");
    let Ok(content) = std::fs::read_to_string(&common_dir_file) else {
        return git_dir.to_path_buf();
    };
    let Some(first_line) = content.lines().next() else {
        return git_dir.to_path_buf();
    };
    let common_dir = PathBuf::from(first_line.trim());
    let resolved = if common_dir.is_absolute() {
        common_dir
    } else {
        git_dir.join(common_dir)
    };
    std::fs::canonicalize(&resolved).unwrap_or(resolved)
}

#[derive(Debug)]
pub struct HookStatus {
    pub hook_type: HookType,
    pub is_installed: bool,
    pub is_managed: bool,
    pub git_runnable: bool,
    pub assura_runnable: bool,
    pub git_path: PathBuf,
    pub assura_path: PathBuf,
}

impl HookStatus {
    pub fn is_ready(&self) -> bool {
        self.is_installed && self.is_managed && self.git_runnable && self.assura_runnable
    }

    pub fn display(&self) -> String {
        let status = if self.is_ready() {
            "✓ installed (managed by assura, runnable)"
        } else if self.is_installed {
            if self.is_managed {
                "⚠ installed (managed by assura, not runnable)"
            } else {
                "⚠ installed (not managed by assura)"
            }
        } else {
            "✗ not installed"
        };

        format!("{:<20} {}", self.hook_type.as_str(), status)
    }
}

#[cfg(unix)]
fn is_runnable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_runnable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_from_str() {
        assert_eq!(
            "pre-commit".parse::<HookType>().unwrap(),
            HookType::PreCommit
        );
        assert!("invalid".parse::<HookType>().is_err());
    }

    #[test]
    fn test_hook_status_display() {
        let status = HookStatus {
            hook_type: HookType::PreCommit,
            is_installed: true,
            is_managed: true,
            git_runnable: true,
            assura_runnable: true,
            git_path: PathBuf::from(".git/hooks/pre-commit"),
            assura_path: PathBuf::from(".assura/hooks/pre-commit"),
        };

        assert!(status.display().contains("installed"));
    }

    #[test]
    fn git_hooks_dir_resolves_regular_git_directory() {
        let project = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();

        let hooks_dir = resolve_git_hooks_dir(project.path()).unwrap();

        assert_eq!(hooks_dir, project.path().join(".git/hooks"));
    }

    #[test]
    fn git_hooks_dir_resolves_worktree_git_file_to_common_hooks() {
        let project = tempfile::TempDir::new().unwrap();
        let git_dir = project.path().join("main.git/worktrees/agent");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::create_dir_all(project.path().join("main.git/hooks")).unwrap();
        std::fs::write(git_dir.join("commondir"), "../..\n# ignored metadata\n").unwrap();
        std::fs::write(
            project.path().join(".git"),
            format!("gitdir: {}\n# ignored metadata\n", git_dir.display()),
        )
        .unwrap();

        let hooks_dir = resolve_git_hooks_dir(project.path()).unwrap();

        assert_eq!(
            hooks_dir,
            std::fs::canonicalize(project.path().join("main.git"))
                .unwrap()
                .join("hooks")
        );
    }
}
