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
pub fn starter_config(
    project_intelligence: bool,
    recipes: &[crate::cli::args::InitRecipe],
    recipe_file: Option<&PathBuf>,
) -> Result<String, StarterInitError> {
    if project_intelligence {
        if !recipes.is_empty() {
            return Err(StarterInitError::Configuration(
                "--project-intelligence cannot currently be combined with --recipe".to_string(),
            ));
        }
        return Ok(project_intelligence_starter_config().to_string());
    }

    let agentic = recipes.contains(&crate::cli::args::InitRecipe::AgenticCore);
    let health = recipes.contains(&crate::cli::args::InitRecipe::StructureHealth);
    let language_recipes = recipes
        .iter()
        .copied()
        .filter(|recipe| {
            matches!(
                recipe,
                crate::cli::args::InitRecipe::RustLibrary
                    | crate::cli::args::InitRecipe::TypescriptBunUtility
                    | crate::cli::args::InitRecipe::PythonPytest
            )
        })
        .collect::<Vec<_>>();
    if !language_recipes.is_empty() {
        if language_recipes.len() != 1 || agentic || health {
            return Err(StarterInitError::Configuration(
                "select exactly one language layout recipe; combine it through an explicit local recipe file instead"
                    .to_string(),
            ));
        }
        return Ok(recipe_config(language_recipes[0]).to_string());
    }
    if agentic || health {
        return Ok(match (agentic, health) {
            (true, true) => AGENTIC_HEALTH_STARTER_CONFIG,
            (true, false) => AGENTIC_CORE_STARTER_CONFIG,
            (false, true) => STRUCTURE_HEALTH_STARTER_CONFIG,
            (false, false) => unreachable!(),
        }
        .to_string());
    }

    let mut config: serde_yaml::Value = serde_yaml::from_str(
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
"#,
    )
    .expect("built-in starter config is valid YAML");
    crate::cli::local_recipe::apply_recipe_file(&mut config, recipe_file)?;
    serde_yaml::to_string(&config).map_err(|error| {
        StarterInitError::Runtime(format!("failed to render starter config: {error}"))
    })
}

/// Return the project-owned YAML fragment for one first-party recipe.
pub fn recipe_config(recipe: crate::cli::args::InitRecipe) -> &'static str {
    match recipe {
        crate::cli::args::InitRecipe::AgenticCore => AGENTIC_CORE_STARTER_CONFIG,
        crate::cli::args::InitRecipe::StructureHealth => STRUCTURE_HEALTH_STARTER_CONFIG,
        crate::cli::args::InitRecipe::RustLibrary => {
            crate::cli::init_recipes::RUST_LIBRARY_STARTER_CONFIG
        }
        crate::cli::args::InitRecipe::TypescriptBunUtility => {
            crate::cli::init_recipes::TYPESCRIPT_BUN_UTILITY_STARTER_CONFIG
        }
        crate::cli::args::InitRecipe::PythonPytest => {
            crate::cli::init_recipes::PYTHON_PYTEST_STARTER_CONFIG
        }
    }
}

/// One project-intelligence starter file materialized by `assura init`.
pub struct StarterFile {
    /// Repository-relative path.
    pub path: &'static str,
    /// Deterministic file contents.
    pub contents: &'static str,
}

/// Error returned while materializing starter files.
#[derive(Debug)]
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

/// Materialize missing support files referenced by selected project-owned recipes.
pub fn materialize_recipe_starter_files(
    project_root: &Path,
    recipes: &[crate::cli::args::InitRecipe],
) -> Result<Vec<PathBuf>, StarterInitError> {
    let mut created = Vec::new();
    for file in recipe_starter_files(recipes) {
        let path = project_root.join(file.path);
        if path.exists() {
            continue;
        }
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
    Ok(created)
}

/// Materialize the selected `assura init` starter files.
pub fn materialize_starter(
    path: Option<PathBuf>,
    force: bool,
    project_intelligence: bool,
    recipes: &[crate::cli::args::InitRecipe],
    recipe_file: Option<PathBuf>,
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
    let config = starter_config(project_intelligence, recipes, recipe_file.as_ref())?;
    crate::config::config::ConfigLoader::parse(&config).map_err(|error| {
        StarterInitError::Configuration(format!("starter recipe is invalid: {error}"))
    })?;
    crate::cli::local_recipe::write_config_atomically(&config_path, &config)?;

    let mut created = vec![config_path];
    if let Some(recipe_file) = recipe_file {
        created.push(crate::cli::local_recipe::write_profile_selection(
            &project_root,
            &recipe_file,
        )?);
    }
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
    } else {
        created.extend(materialize_recipe_starter_files(&project_root, recipes)?);
    }

    Ok(created)
}

fn recipe_starter_files(recipes: &[crate::cli::args::InitRecipe]) -> Vec<&'static StarterFile> {
    let mut files = Vec::new();
    if recipes.contains(&crate::cli::args::InitRecipe::AgenticCore) {
        files.extend(AGENTIC_CORE_STARTER_FILES);
    }
    if recipes.contains(&crate::cli::args::InitRecipe::StructureHealth) {
        files.extend(STRUCTURE_HEALTH_STARTER_FILES);
    }
    files
}

const AGENTIC_CORE_STARTER_CONFIG: &str = r#"rules:
  agent-entrypoint:
    max_lines: 160
    severity: low
    message: See docs/agent-guidance.md.

  skill-entrypoint:
    max_lines: 500
    markdown:
      require_frontmatter: true
    message: See docs/agent-guidance.md#skills.

  closed-entry:
    exists: 0
    message: See docs/agent-guidance.md#layout.

  closed:
    ./*/: $closed-entry
    ./*: $closed-entry

  skill:
    ./: $closed
    ./{agents,assets,references,scripts}/:
      ./: exists:0-1
      inherit: false
    SKILL.md: exists:1 | $skill-entrypoint

structure:
  .agents/:
    ./: exists:0-1 | $closed
    skills/:
      ./: exists:0-1
      ./*/: kebab-case | $skill
      ./*: $closed-entry

  AGENTS.md: exists:1 | $agent-entrypoint
  README.md: exists:1

exclude:
  - "**/{.git,node_modules,target,dist,coverage}/**"
"#;

const STRUCTURE_HEALTH_STARTER_CONFIG: &str = r#"rules:
  folder-health:
    limit_children: 10
    severity: low
    message: See docs/structure.md.

structure:
  ./**/:
    ./: $folder-health
    .{md,js,jsx,ts,tsx}: max_lines:500 | severity:low

exclude:
  - "**/{.git,node_modules,target,dist,coverage}/**"
"#;

const AGENTIC_HEALTH_STARTER_CONFIG: &str = r#"rules:
  agent-entrypoint:
    max_lines: 160
    severity: low
    message: See docs/agent-guidance.md.

  skill-entrypoint:
    max_lines: 500
    markdown:
      require_frontmatter: true
    message: See docs/agent-guidance.md#skills.

  folder-health:
    limit_children: 10
    severity: low
    message: See docs/structure.md.

  closed-entry:
    exists: 0
    message: See docs/agent-guidance.md#layout.

  closed:
    ./*/: $closed-entry
    ./*: $closed-entry

  skill:
    ./: $closed
    ./{agents,assets,references,scripts}/:
      ./: exists:0-1
      inherit: false
    SKILL.md: exists:1 | $skill-entrypoint

structure:
  .agents/:
    ./: exists:0-1 | $closed
    skills/:
      ./: exists:0-1
      ./*/: kebab-case | $skill
      ./*: $closed-entry

  AGENTS.md: exists:1 | $agent-entrypoint
  README.md: exists:1

  ./**/:
    ./: $folder-health
    .{md,js,jsx,ts,tsx}: max_lines:500 | severity:low

exclude:
  - "**/{.git,node_modules,target,dist,coverage}/**"
"#;

const AGENTIC_CORE_STARTER_FILES: &[&StarterFile] = &[
    &StarterFile {
        path: "README.md",
        contents: "# Project\n",
    },
    &StarterFile {
        path: "AGENTS.md",
        contents: r#"# Agent Guidance

Read [docs/agent-guidance.md](docs/agent-guidance.md) before changing project
structure or adding project-local skills.
"#,
    },
    &StarterFile {
        path: "docs/agent-guidance.md",
        contents: r#"# Agent Guidance

Keep root and package guidance concise. Link to durable project documentation
instead of copying large instructions into every agent context.

## Skills

Each directory under `.agents/skills/` must contain `SKILL.md`. Keep detailed
examples and references beside that entrypoint for progressive disclosure.

## Layout

Add new paths to the project-owned Assura policy before creating them. Prefer an
existing directory when it already owns the concern.
"#,
    },
];

const STRUCTURE_HEALTH_STARTER_FILES: &[&StarterFile] = &[&StarterFile {
    path: "docs/structure.md",
    contents: r#"# Project Structure

When a directory approaches the configured direct-child limit, group related
files by responsibility while the change is still small. Update the project-owned
Assura policy when a larger directory is intentional.
"#,
}];

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
