//! Helpers for initializing a project with a starter Assura config.

use std::path::{Path, PathBuf};

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
pub fn starter_config(project_intelligence: bool) -> &'static str {
    if project_intelligence {
        return project_intelligence_starter_config();
    }

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
    exists: 0-1
    .rs: snake_case
  tests/:
    exists: 0-1
    .rs: snake_case

exclude:
  - ".git/**"
  - "target/**"
  - "node_modules/**"
  - "dist/**"
  - "**/dist/**"
"#
}

/// One project-intelligence starter file materialized by `assura init`.
pub struct StarterFile {
    /// Repository-relative path.
    pub path: &'static str,
    /// Deterministic file contents.
    pub contents: &'static str,
}

/// Error returned while materializing starter files.
pub enum StarterInitError {
    /// User-correctable configuration or overwrite problem.
    Configuration(String),
    /// Filesystem or environment failure.
    Runtime(String),
}

impl StarterInitError {
    /// User-facing error message.
    pub fn message(&self) -> &str {
        match self {
            Self::Configuration(message) | Self::Runtime(message) => message,
        }
    }
}

/// Starter files for the project-intelligence onboarding profile.
pub fn project_intelligence_starter_files() -> &'static [StarterFile] {
    PROJECT_INTELLIGENCE_STARTER_FILES
}

/// Materialize the selected `assura init` starter files.
pub fn materialize_starter(
    path: Option<PathBuf>,
    force: bool,
    project_intelligence: bool,
) -> Result<Vec<PathBuf>, StarterInitError> {
    let project_root =
        resolve_project_root(path).map_err(|error| StarterInitError::Runtime(error.to_string()))?;
    let assura_dir = project_root.join(".assura");
    let config_path = assura_dir.join("config.yml");
    if config_path.exists() && !force {
        return Err(StarterInitError::Configuration(format!(
            "{} already exists. Use --force to overwrite.",
            config_path.display()
        )));
    }
    if project_intelligence && !force {
        if let Some(path) = existing_project_intelligence_starter_paths(&project_root).first() {
            return Err(StarterInitError::Configuration(format!(
                "{} already exists. Use --force to overwrite project-intelligence starter files.",
                path.display()
            )));
        }
    }

    std::fs::create_dir_all(&assura_dir).map_err(|error| {
        StarterInitError::Runtime(format!(
            "failed to create {}: {}",
            assura_dir.display(),
            error
        ))
    })?;
    std::fs::write(&config_path, starter_config(project_intelligence)).map_err(|error| {
        StarterInitError::Runtime(format!(
            "failed to write {}: {}",
            config_path.display(),
            error
        ))
    })?;

    let mut created = vec![config_path];
    if project_intelligence {
        for file in project_intelligence_starter_files() {
            let path = project_root.join(file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    StarterInitError::Runtime(format!(
                        "failed to create {}: {}",
                        parent.display(),
                        error
                    ))
                })?;
            }
            std::fs::write(&path, file.contents).map_err(|error| {
                StarterInitError::Runtime(format!("failed to write {}: {}", path.display(), error))
            })?;
            created.push(path);
        }
    }

    Ok(created)
}

/// Check whether any project-intelligence starter file would be overwritten.
pub fn existing_project_intelligence_starter_paths(project_root: &Path) -> Vec<PathBuf> {
    project_intelligence_starter_files()
        .iter()
        .map(|file| project_root.join(file.path))
        .filter(|path| path.exists())
        .collect()
}

fn project_intelligence_starter_config() -> &'static str {
    r#"version: "2.0"

structure:
  ./:
    extra: true
    README.md: exists:0-1
    LICENSE: exists:0-1
    ".gitignore": exists:0-1
    Cargo.toml: exists:0-1
    package.json: exists:0-1
    docs/: exists:0-1
    specs/: exists:0-1
    src/: exists:0-1
    tests/: exists:0-1
    .rs: snake_case
  docs/goals/:
    exists: 0-1
    .md:
      markdown:
        lint_trailing_spaces: true
  src/:
    exists: 0-1
    .rs: snake_case
  tests/:
    exists: 0-1
    .rs: snake_case

models:
  validation_artifact: .assura/models/project-intelligence/starter.schema.json

collections:
  goals:
    class: Goal
    path: docs/goals/*.md
    adapter: markdown_frontmatter
    data: frontmatter
    body: markdown
    id: id
  specs:
    class: Spec
    path: specs/*.json
    adapter: json_record
    id: id
  decisions:
    class: Decision
    path: docs/decisions/*.json
    adapter: json_record
    id: id

relations:
  goals.specs:
    target: specs
    many: true
    required: true
  goals.decisions:
    target: decisions
    many: true
    required: true

exclude:
  - ".git/**"
  - "target/**"
  - "node_modules/**"
  - "dist/**"
  - "**/dist/**"
"#
}

const PROJECT_INTELLIGENCE_STARTER_FILES: &[StarterFile] = &[
    StarterFile {
        path: ".assura/models/project-intelligence/starter.schema.json",
        contents: r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://assura.dev/starters/project-intelligence.schema.json",
  "$defs": {
    "Goal": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "title", "status", "owner", "specs", "decisions"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "type": "string", "enum": ["planned", "active", "done"] },
        "owner": { "type": "string", "minLength": 1 },
        "specs": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        },
        "decisions": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }
      }
    },
    "Spec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "title", "status"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "type": "string", "enum": ["draft", "active", "done"] }
      }
    },
    "Decision": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "title", "status"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "type": "string", "enum": ["proposed", "accepted", "superseded"] }
      }
    }
  }
}
"#,
    },
    StarterFile {
        path: "docs/goals/goal_project_intelligence_starter.md",
        contents: r#"---
id: goal-project-intelligence-starter
title: Adopt Project Intelligence
status: active
owner: maintainers
specs:
  - spec-project-intelligence-starter
decisions:
  - adr-project-intelligence-starter
---

# Adopt Project Intelligence

Use this goal as the first modeled project object. Replace it with a real
goal, spec, ADR, package, or release workflow when the starter proves useful.

## First Query

Search for "starter project context" to verify modeled content and Markdown
sections are available to Assura.
"#,
    },
    StarterFile {
        path: "specs/spec_project_intelligence_starter.json",
        contents: r#"{
  "id": "spec-project-intelligence-starter",
  "title": "Project intelligence starter context",
  "status": "active"
}
"#,
    },
    StarterFile {
        path: "docs/decisions/adr_project_intelligence_starter.json",
        contents: r#"{
  "id": "adr-project-intelligence-starter",
  "title": "Keep project intelligence local first",
  "status": "accepted"
}
"#,
    },
    StarterFile {
        path: "docs/examples/project-intelligence-broken-goal.md",
        contents: r#"---
id: goal-project-intelligence-missing-context
title: Broken Project Intelligence Example
status: active
owner: maintainers
specs:
  - missing-spec-project-context
decisions:
  - adr-project-intelligence-starter
---

# Broken Project Intelligence Example

Copy this file into `docs/goals/` to see
`content_runtime:missing_reference` diagnostics for a missing spec relation.
"#,
    },
];
