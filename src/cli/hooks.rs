//! Git hook installation and status management for Assura.
use std::path::{Path, PathBuf};

use thiserror::Error;

#[path = "hooks_ownership.rs"]
mod ownership;
#[path = "hooks_status.rs"]
mod status;
#[path = "hooks_transaction.rs"]
mod transaction;

pub use status::HookStatus;

use ownership::{
    classify_artifact, ensure_plain_directory, plain_directory_or_absent,
    remove_file_if_still_managed, shell_single_quote, ArtifactOwnership, HookOwnership,
};
use status::is_runnable;

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

    #[error("Refusing hook mutation through unmanaged path: {0}")]
    UnsafePath(PathBuf),

    #[error("Hook mutation failed: {operation}; rollback also failed: {rollback}")]
    RollbackFailed { operation: String, rollback: String },
}

pub type HookResult<T> = Result<T, HookError>;

/// Outcome of installing the managed Git hooks for a project.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HookInstallOutcome {
    /// Hook entrypoints created by Assura during this invocation.
    pub installed: Vec<HookType>,
    /// Managed hook entrypoints that already matched the current generator.
    pub unchanged: Vec<HookType>,
    /// Managed hook entrypoints refreshed because they were stale or force was explicit.
    pub refreshed: Vec<HookType>,
    /// Hook artifact pairs Assura left unchanged because ownership was not proven.
    pub preserved: Vec<HookType>,
}

/// Outcome of removing Assura-managed Git hooks from a project.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HookUninstallOutcome {
    /// Managed hook entrypoints removed by Assura.
    pub removed: Vec<HookType>,
    /// Hook artifact pairs Assura left unchanged because ownership was not proven.
    pub preserved: Vec<HookType>,
}

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

    pub fn install_all(&self, force: bool) -> HookResult<HookInstallOutcome> {
        let mut outcome = HookInstallOutcome::default();

        for hook_type in HookType::all() {
            let ownership = self.ownership(hook_type)?;
            if ownership.has_unmanaged() {
                outcome.preserved.push(hook_type);
                continue;
            }

            if ownership.is_current() && !force {
                outcome.unchanged.push(hook_type);
                continue;
            }

            match self.install(hook_type, force) {
                Ok(_) if ownership.has_managed_artifact() => outcome.refreshed.push(hook_type),
                Ok(_) => outcome.installed.push(hook_type),
                Err(e) => return Err(e),
            }
        }

        Ok(outcome)
    }

    pub fn install(&self, hook_type: HookType, force: bool) -> HookResult<()> {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);
        let ownership = self.ownership(hook_type)?;

        if ownership.has_unmanaged() || (ownership.is_current() && !force) {
            return Err(HookError::AlreadyExists(hook_name.to_string()));
        }

        self.ensure_mutation_directories()?;

        // Reclassify at the final mutation boundary so a static path change made
        // after manager creation cannot turn a managed write into an overwrite.
        let ownership = self.ownership(hook_type)?;
        if ownership.has_unmanaged() || (ownership.is_current() && !force) {
            return Err(HookError::AlreadyExists(hook_name.to_string()));
        }

        let hook_content = self.generate_hook_content(hook_type)?;
        let git_hook_content = self.managed_git_hook_content(&assura_hook_path);
        transaction::replace_pair(
            &assura_hook_path,
            hook_content.as_bytes(),
            &git_hook_path,
            &git_hook_content,
        )
    }

    fn managed_git_hook_content(&self, assura_hook_path: &Path) -> Vec<u8> {
        let mut content = br#"#!/bin/sh
# Git hook managed by Assura
# This file was auto-generated. Do not modify manually.

ASSURA_HOOK="#
            .to_vec();
        content.extend_from_slice(&shell_single_quote(assura_hook_path));
        content.extend_from_slice(
            br#"

if [ -f "$ASSURA_HOOK" ]; then
    exec "$ASSURA_HOOK" "$@"
else
    echo "Warning: Assura hook not found at $ASSURA_HOOK" >&2
    exit 0
fi
"#,
        );
        content
    }

    fn legacy_managed_git_hook_content(&self, assura_hook_path: &Path) -> Option<Vec<u8>> {
        let assura_hook_path = assura_hook_path.to_str()?;
        Some(
            format!(
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
                assura_hook_path
            )
            .into_bytes(),
        )
    }

    pub fn uninstall(&self, hook_type: HookType) -> HookResult<()> {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);

        let ownership = self.ownership(hook_type)?;
        if ownership.has_unmanaged() {
            return Ok(());
        }

        if ownership.wrapper.is_managed() {
            let expected = self.managed_wrapper_contents(&assura_hook_path);
            remove_file_if_still_managed(&git_hook_path, &expected)?;
        }
        if ownership.sidecar.is_managed() {
            let expected = self.generate_hook_content(hook_type)?.into_bytes();
            remove_file_if_still_managed(&assura_hook_path, &[expected])?;
        }

        Ok(())
    }

    pub fn uninstall_all(&self) -> HookResult<HookUninstallOutcome> {
        let mut outcome = HookUninstallOutcome::default();

        for hook_type in HookType::all() {
            let ownership = self.ownership(hook_type)?;
            if ownership.has_unmanaged() {
                outcome.preserved.push(hook_type);
            } else if ownership.has_managed_artifact() {
                self.uninstall(hook_type)?;
                outcome.removed.push(hook_type);
            }
        }

        Ok(outcome)
    }

    pub fn status(&self, hook_type: HookType) -> HookStatus {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);

        let ownership = self.ownership(hook_type).unwrap_or(HookOwnership {
            wrapper: ArtifactOwnership::Unmanaged,
            sidecar: ArtifactOwnership::Unmanaged,
        });

        HookStatus {
            hook_type,
            is_installed: ownership.is_installed(),
            is_managed: ownership.is_complete(),
            is_current: ownership.is_current(),
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

    fn ownership(&self, hook_type: HookType) -> HookResult<HookOwnership> {
        let hook_name = hook_type.as_str();
        let git_hook_path = self.git_hooks_dir.join(hook_name);
        let assura_hook_path = self.assura_hooks_dir.join(hook_name);
        let wrapper_expected = self.managed_wrapper_contents(&assura_hook_path);
        let sidecar_expected = self.generate_hook_content(hook_type)?.into_bytes();

        Ok(HookOwnership {
            wrapper: classify_artifact(
                &git_hook_path,
                &wrapper_expected,
                plain_directory_or_absent(&self.git_hooks_dir),
            )?,
            sidecar: classify_artifact(
                &assura_hook_path,
                &[sidecar_expected],
                self.assura_hook_directories_are_safe(),
            )?,
        })
    }

    fn managed_wrapper_contents(&self, assura_hook_path: &Path) -> Vec<Vec<u8>> {
        let mut contents = vec![self.managed_git_hook_content(assura_hook_path)];
        if let Some(legacy) = self.legacy_managed_git_hook_content(assura_hook_path) {
            contents.push(legacy);
        }
        contents
    }

    fn assura_hook_directories_are_safe(&self) -> bool {
        self.assura_hooks_dir
            .parent()
            .is_some_and(plain_directory_or_absent)
            && plain_directory_or_absent(&self.assura_hooks_dir)
    }

    fn ensure_mutation_directories(&self) -> HookResult<()> {
        ensure_plain_directory(&self.git_hooks_dir)?;
        let assura_dir = self
            .assura_hooks_dir
            .parent()
            .ok_or_else(|| HookError::UnsafePath(self.assura_hooks_dir.clone()))?;
        ensure_plain_directory(assura_dir)?;
        ensure_plain_directory(&self.assura_hooks_dir)
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
            is_current: true,
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
    fn legacy_wrapper_bytes_match_the_historical_fixture_shape() {
        let project = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();
        let manager = GitHooksManager::new(project.path()).unwrap();
        let sidecar = project.path().join(".assura/hooks/pre-push");
        let fixture = format!(
            "#!/bin/sh\n# Git hook managed by Assura\n# This file was auto-generated. Do not modify manually.\n\nASSURA_HOOK=\"{}\"\n\nif [ -f \"$ASSURA_HOOK\" ]; then\n    exec \"$ASSURA_HOOK\" \"$@\"\nelse\n    echo \"Warning: Assura hook not found at $ASSURA_HOOK\" >&2\n    exit 0\nfi\n",
            sidecar.display()
        );

        assert_eq!(
            manager.legacy_managed_git_hook_content(&sidecar).unwrap(),
            fixture.into_bytes()
        );
    }

    #[test]
    fn legacy_wrapper_fixture_is_classified_after_current_installation() {
        let project = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();
        std::fs::create_dir_all(project.path().join(".assura/hooks")).unwrap();
        let manager = GitHooksManager::new(project.path()).unwrap();
        manager.install(HookType::PrePush, false).unwrap();
        let wrapper = project.path().join(".git/hooks/pre-push");
        let sidecar = project.path().join(".assura/hooks/pre-push");
        let fixture = format!(
            "#!/bin/sh\n# Git hook managed by Assura\n# This file was auto-generated. Do not modify manually.\n\nASSURA_HOOK=\"{}\"\n\nif [ -f \"$ASSURA_HOOK\" ]; then\n    exec \"$ASSURA_HOOK\" \"$@\"\nelse\n    echo \"Warning: Assura hook not found at $ASSURA_HOOK\" >&2\n    exit 0\nfi\n",
            sidecar.display()
        );
        std::fs::write(&wrapper, fixture).unwrap();

        let ownership = manager.ownership(HookType::PrePush).unwrap();
        assert_eq!(ownership.wrapper, ArtifactOwnership::ManagedLegacy);
        assert_eq!(ownership.sidecar, ArtifactOwnership::ManagedCurrent);
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
