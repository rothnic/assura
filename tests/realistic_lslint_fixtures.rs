#![allow(dead_code)]

use assura::config::ls_compat::convert_ls_lint_to_config;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::{Builder, TempDir};

pub const MANIFEST: &str = include_str!("ls_lint_realistic_fixture_manifest.yml");

#[derive(Debug, Deserialize)]
pub struct FixtureManifest {
    pub schema_version: u64,
    pub fixtures: Vec<FixtureManifestEntry>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureManifestEntry {
    pub id: String,
    pub name: String,
    pub source: FixtureSource,
    pub purpose: String,
    pub ls_lint_rules: Vec<String>,
    pub assura_rules: Vec<String>,
    pub cohort: String,
    pub native_lslint_parity: bool,
    pub assura_extensions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureSource {
    pub kind: String,
    pub repository: String,
    pub revision: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureVariant {
    Valid,
    Invalid,
}

#[derive(Clone, Copy, Debug)]
pub struct FixtureFamily {
    pub id: &'static str,
    pub valid_expected_rules: &'static [&'static str],
    pub invalid_expected_rules: &'static [&'static str],
}

pub struct MaterializedFixture {
    pub project: TempDir,
    pub family_id: &'static str,
    pub variant: FixtureVariant,
    pub expected_rules: &'static [&'static str],
}

const FAMILIES: &[FixtureFamily] = &[
    FixtureFamily {
        id: "simple_library",
        valid_expected_rules: &[],
        invalid_expected_rules: &["directory_naming", "exists_count", "file_naming"],
    },
    FixtureFamily {
        id: "web_app",
        valid_expected_rules: &[],
        invalid_expected_rules: &["directory_naming", "file_naming"],
    },
    FixtureFamily {
        id: "monorepo_packages",
        valid_expected_rules: &[],
        invalid_expected_rules: &["directory_naming", "exists_count", "file_naming"],
    },
    FixtureFamily {
        id: "monorepo_policy",
        valid_expected_rules: &[],
        invalid_expected_rules: &["file_naming", "unexpected_directory", "unexpected_file"],
    },
    FixtureFamily {
        id: "rule_heavy_repo",
        valid_expected_rules: &[],
        invalid_expected_rules: &["file_naming"],
    },
    FixtureFamily {
        id: "ignored_generated_heavy_repo",
        valid_expected_rules: &[],
        invalid_expected_rules: &["file_naming"],
    },
];

pub fn parse_manifest() -> FixtureManifest {
    serde_yaml::from_str(MANIFEST).expect("fixture manifest should parse")
}

pub fn realistic_fixture_families() -> &'static [FixtureFamily] {
    FAMILIES
}

pub fn materialize_fixture(family: FixtureFamily, variant: FixtureVariant) -> MaterializedFixture {
    let project = Builder::new()
        .prefix(&format!("assura_realistic_{}_", family.id))
        .tempdir()
        .unwrap();

    match family.id {
        "simple_library" => simple_library(&project, variant),
        "web_app" => web_app(&project, variant),
        "monorepo_packages" => monorepo_packages(&project, variant),
        "monorepo_policy" => monorepo_policy(&project, variant),
        "rule_heavy_repo" => rule_heavy_repo(&project, variant),
        "ignored_generated_heavy_repo" => ignored_generated_heavy_repo(&project, variant),
        unknown => panic!("unknown fixture family: {unknown}"),
    }

    MaterializedFixture {
        project,
        family_id: family.id,
        variant,
        expected_rules: match variant {
            FixtureVariant::Valid => family.valid_expected_rules,
            FixtureVariant::Invalid => family.invalid_expected_rules,
        },
    }
}

pub fn materialize_external_git_fixture(
    entry: &FixtureManifestEntry,
    cache_root: &Path,
    destination: &Path,
) -> Result<(), String> {
    if entry.source.kind != "external_git" {
        return Err(format!(
            "fixture {} is {}, not external_git",
            entry.id, entry.source.kind
        ));
    }

    fs::create_dir_all(cache_root)
        .map_err(|error| format!("create cache {}: {error}", cache_root.display()))?;

    let cache_dir = cache_root.join(cache_key(&entry.source.repository, &entry.source.revision));
    if !cache_dir.exists() {
        run_git(
            [
                "clone",
                "--no-checkout",
                "--quiet",
                &entry.source.repository,
                cache_dir.to_str().ok_or("cache path is not valid UTF-8")?,
            ],
            None,
        )?;
    } else {
        run_git(["fetch", "--tags", "--quiet", "origin"], Some(&cache_dir))?;
    }

    run_git(
        ["checkout", "--quiet", &entry.source.revision],
        Some(&cache_dir),
    )?;
    let checked_out = git_output(["rev-parse", "HEAD"], Some(&cache_dir))?;
    if !checked_out.starts_with(&entry.source.revision) && checked_out != entry.source.revision {
        return Err(format!(
            "fixture {} checked out {}, expected {}",
            entry.id, checked_out, entry.source.revision
        ));
    }

    if destination.exists() {
        fs::remove_dir_all(destination)
            .map_err(|error| format!("clear destination {}: {error}", destination.display()))?;
    }
    fs::create_dir_all(destination)
        .map_err(|error| format!("create destination {}: {error}", destination.display()))?;
    copy_without_git(&cache_dir, destination)
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

fn write_configs(project: &TempDir, ls_lint_config: &str) {
    let config = convert_ls_lint_to_config(ls_lint_config).unwrap();
    write_assura_and_lslint_configs(
        project,
        &serde_yaml::to_string(&config).unwrap(),
        ls_lint_config,
    );
}

fn write_assura_and_lslint_configs(project: &TempDir, assura_config: &str, ls_lint_config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), assura_config).unwrap();
    fs::write(project.path().join(".ls-lint.yml"), ls_lint_config).unwrap();
}

fn write(path: impl AsRef<Path>, content: &str) {
    fs::write(path, content).unwrap();
}

fn simple_library(project: &TempDir, variant: FixtureVariant) {
    write_configs(
        project,
        r#"
ignore:
  - .assura/**
  - target/**
ls:
  .dir: kebab-case
  .rs: snake_case
  .md: exists:1-3
  .test.ts: kebab-case
  src:
    .rs: snake_case
  tests:
    .rs: snake_case
"#,
    );

    write(project.path().join("README.md"), "# Library\n");
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("tests")).unwrap();
    fs::create_dir(project.path().join("target")).unwrap();
    write(project.path().join("src/lib.rs"), "pub fn library() {}\n");
    write(
        project.path().join("tests/smoke_test.rs"),
        "#[test] fn ok() {}\n",
    );
    write(project.path().join("client-api.test.ts"), "export {};\n");
    write(project.path().join("target/BadName.rs"), "");

    if variant == FixtureVariant::Invalid {
        fs::create_dir(project.path().join("BadDir")).unwrap();
        write(project.path().join("src/bad-name.rs"), "");
        write(project.path().join("BadName.test.ts"), "");
        write(project.path().join("extra.md"), "# Extra\n");
        write(project.path().join("another.md"), "# Another\n");
        write(project.path().join("overflow.md"), "# Overflow\n");
    }
}

fn web_app(project: &TempDir, variant: FixtureVariant) {
    write_configs(
        project,
        r#"
ignore:
  - .assura/**
  - dist/**
ls:
  .dir: kebab-case
  .tsx: PascalCase
  .test.tsx: kebab-case
  .module.css: kebab-case
  .png: kebab-case
  src:
    .dir: kebab-case
    .tsx: PascalCase
    .test.tsx: kebab-case
  public:
    .png: kebab-case
"#,
    );

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("public")).unwrap();
    fs::create_dir(project.path().join("dist")).unwrap();
    write(
        project.path().join("src/App.tsx"),
        "export function App() {}\n",
    );
    write(project.path().join("src/app.test.tsx"), "export {};\n");
    write(project.path().join("src/theme.module.css"), ".root {}\n");
    write(project.path().join("public/logo-icon.png"), "");
    write(project.path().join("dist/BadName.tsx"), "");

    if variant == FixtureVariant::Invalid {
        fs::create_dir(project.path().join("src/BadDir")).unwrap();
        write(project.path().join("src/bad-widget.tsx"), "export {};\n");
        write(
            project.path().join("src/BadWidget.test.tsx"),
            "export {};\n",
        );
        write(
            project.path().join("src/ThemeStyles.module.css"),
            ".root {}\n",
        );
    }
}

fn monorepo_packages(project: &TempDir, variant: FixtureVariant) {
    write_configs(
        project,
        r#"
ignore:
  - .assura/**
  - packages/core/dist/**
  - packages/ui/dist/**
ls:
  .dir: kebab-case
  packages:
    core:
      .dir: kebab-case
      .ts: camelCase
      .md: exists:1-2
      src:
        .ts: camelCase
      tests:
        .test.ts: kebab-case
    ui:
      .dir: kebab-case
      .tsx: PascalCase
      .md: exists:1-2
      src:
        .tsx: PascalCase
      tests:
        .test.tsx: kebab-case
"#,
    );

    for package in ["core", "ui"] {
        fs::create_dir_all(project.path().join(format!("packages/{package}/src"))).unwrap();
        fs::create_dir(project.path().join(format!("packages/{package}/tests"))).unwrap();
        fs::create_dir(project.path().join(format!("packages/{package}/dist"))).unwrap();
        write(
            project.path().join(format!("packages/{package}/README.md")),
            "# Package\n",
        );
    }
    write(project.path().join("packages/core/src/indexFile.ts"), "");
    write(
        project
            .path()
            .join("packages/core/tests/index-file.test.ts"),
        "",
    );
    write(project.path().join("packages/core/dist/BadName.ts"), "");
    write(project.path().join("packages/ui/src/Button.tsx"), "");
    write(project.path().join("packages/ui/tests/button.test.tsx"), "");
    write(project.path().join("packages/ui/dist/bad-name.tsx"), "");

    if variant == FixtureVariant::Invalid {
        fs::create_dir(project.path().join("packages/core/BadDir")).unwrap();
        write(project.path().join("packages/core/src/bad-name.ts"), "");
        write(project.path().join("packages/core/extra.md"), "# Extra\n");
        write(
            project.path().join("packages/core/overflow.md"),
            "# Overflow\n",
        );
        write(project.path().join("packages/ui/src/button.tsx"), "");
    }
}

fn monorepo_policy(project: &TempDir, variant: FixtureVariant) {
    write_assura_and_lslint_configs(
        project,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
        - AGENTS.md
        - package.json
      allow_extra: false
    directories:
      allowed_names:
        - apps
        - packages
        - docs
        - generated
        - node_modules
      allow_extra: false
    children:
      apps:
        children:
          web:
            files:
              naming_patterns:
                "*.js": regex:(next\.config|postcss\.config|tailwind\.config)
                "*.json": kebab-case
            directories:
              naming: kebab-case
            children:
              src:
                files:
                  naming_patterns:
                    "*.ts": kebab-case
                    "*.tsx": PascalCase
                    "*.js": regex:^$
      packages:
        children:
          core:
            children:
              src:
                files:
                  naming_patterns:
                    "*.ts": kebab-case
                    "*.js": regex:^$
      docs:
        files:
          naming_patterns:
            "*.md": regex:(README|AGENTS) | kebab-case
exclude:
  - ".assura/**"
  - ".ls-lint.yml"
  - "generated/**"
  - "node_modules/**"
"#,
        r#"
ignore:
  - .assura/**
  - .ls-lint.yml
  - generated/**
  - node_modules/**
ls:
  .dir: regex:(apps|packages|docs|generated|node_modules)
  .md: regex:(README|AGENTS) | kebab-case
  .json: kebab-case
  apps:
    web:
      .js: regex:(next\.config|postcss\.config|tailwind\.config)
      .json: kebab-case
      src:
        .ts: kebab-case
        .tsx: PascalCase
        .js: regex:^$
  packages:
    core:
      src:
        .ts: kebab-case
        .js: regex:^$
  docs:
    .md: regex:(README|AGENTS) | kebab-case
"#,
    );

    for dir in [
        "apps/web/src/components",
        "packages/core/src",
        "docs",
        "generated/api",
        "node_modules/pkg",
    ] {
        fs::create_dir_all(project.path().join(dir)).unwrap();
    }
    for file in [
        "README.md",
        "AGENTS.md",
        "package.json",
        "apps/web/package.json",
        "apps/web/next.config.js",
        "apps/web/src/index.ts",
        "apps/web/src/components/DashboardShell.tsx",
        "packages/core/src/index.ts",
        "docs/README.md",
        "docs/architecture-notes.md",
        "generated/api/BadName.js",
        "node_modules/pkg/BadName.js",
    ] {
        write(project.path().join(file), "");
    }

    if variant == FixtureVariant::Invalid {
        fs::create_dir(project.path().join("scratch")).unwrap();
        write(project.path().join("notes.md"), "");
        write(
            project
                .path()
                .join("apps/web/src/components/bad-component.tsx"),
            "",
        );
        write(project.path().join("apps/web/src/legacy.js"), "");
    }
}

fn rule_heavy_repo(project: &TempDir, variant: FixtureVariant) {
    let mut rules = String::new();
    for index in 0..36 {
        rules.push_str(&format!("  .kind-{index:02}.ts: kebab-case\n"));
    }
    write_configs(
        project,
        &format!(
            r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .dir: kebab-case
  .*: kebab-case | snake_case
{rules}"#
        ),
    );

    for dir_index in 0..8 {
        let dir = project.path().join(format!("feature-{dir_index:02}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..24 {
            let kind = file_index % 36;
            write(
                dir.join(format!(
                    "feature-{dir_index:02}-{file_index:02}.kind-{kind:02}.ts"
                )),
                "",
            );
        }
    }

    if variant == FixtureVariant::Invalid {
        write(project.path().join("feature-00/BadName.kind-00.ts"), "");
    }
}

fn ignored_generated_heavy_repo(project: &TempDir, variant: FixtureVariant) {
    write_configs(
        project,
        r#"
ignore:
  - .assura/**
  - generated/**
  - coverage/**
ls:
  .dir: kebab-case
  .ts: kebab-case
"#,
    );

    fs::create_dir(project.path().join("src")).unwrap();
    write(project.path().join("src/index-file.ts"), "");
    for root in ["generated", "coverage"] {
        for dir_index in 0..24 {
            let dir = project.path().join(format!("{root}/out-{dir_index:02}"));
            fs::create_dir_all(&dir).unwrap();
            for file_index in 0..16 {
                write(dir.join(format!("BAD_{file_index:02}.ts")), "");
            }
        }
    }

    if variant == FixtureVariant::Invalid {
        write(project.path().join("src/BadName.ts"), "");
    }
}
