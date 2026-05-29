//! Realistic generated fixtures shared by performance report scenarios.

use super::fixture_io::{write_file, write_lslint_compatible_configs};
use std::fs;
use std::path::Path;

pub(super) fn create_monorepo_packages_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
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
    )?;
    for package in ["core", "ui"] {
        fs::create_dir_all(root.join(format!("packages/{package}/src")))
            .map_err(|error| format!("create package src: {error}"))?;
        fs::create_dir(root.join(format!("packages/{package}/tests")))
            .map_err(|error| format!("create package tests: {error}"))?;
        fs::create_dir(root.join(format!("packages/{package}/dist")))
            .map_err(|error| format!("create package dist: {error}"))?;
        write_file(
            root.join(format!("packages/{package}/README.md")),
            "# Package\n",
        )?;
    }
    write_file(root.join("packages/core/src/indexFile.ts"), "")?;
    write_file(root.join("packages/core/tests/index-file.test.ts"), "")?;
    write_file(root.join("packages/core/dist/BadName.ts"), "")?;
    write_file(root.join("packages/ui/src/Button.tsx"), "")?;
    write_file(root.join("packages/ui/tests/button.test.tsx"), "")?;
    write_file(root.join("packages/ui/dist/bad-name.tsx"), "")?;
    Ok(())
}
