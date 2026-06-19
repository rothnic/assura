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
    extra: true
    README.md: exists:0-1
    LICENSE: exists:0-1
    ".gitignore": exists:0-1
    Cargo.toml: exists:0-1
    package.json: exists:0-1
    src/: exists:0-1
    tests/: exists:0-1
    .rs: snake_case
  src/:
    required: false
    .rs: snake_case
  tests/:
    required: false
    .rs: snake_case

exclude:
  - ".git/**"
  - "target/**"
  - "node_modules/**"
  - "dist/**"
  - "**/dist/**"
"#
}
