//! Helpers for initializing a project with a starter Assura config.

use std::path::PathBuf;

/// Resolve the project root used by `assura init`.
pub fn resolve_project_root(path: Option<PathBuf>) -> std::io::Result<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };

    if path.exists() {
        path.canonicalize()
    } else {
        Ok(path)
    }
}

/// Starter structure config written by `assura init`.
pub fn starter_config() -> &'static str {
    r#"version: "2.0"

structure:
  ./:
    files:
      naming: "kebab-case"
      allowed_names:
        - "README.md"
        - "AGENTS.md"
        - "LICENSE"
        - ".gitignore"
        - "Cargo.toml"
        - "package.json"
      allow_extra: true
    directories:
      naming: "kebab-case"
      allowed_names:
        - ".assura"
      allow_extra: true
    children:
      .assura/:
        inherit: false
        files:
          naming: "kebab-case"

exclude:
  - ".git/**"
  - "target/**"
  - "node_modules/**"
  - "dist/**"
  - "**/dist/**"
"#
}
