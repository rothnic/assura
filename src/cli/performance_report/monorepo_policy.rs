//! Rich generated monorepo-policy fixture for performance reports.

use super::fixture_io::{write_configs, write_file, write_lslint_compatible_configs};
use std::fs;
use std::path::Path;

pub(super) fn create_monorepo_policy_project(root: &Path) -> Result<(), String> {
    write_configs(
        root,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
        - AGENTS.md
        - CONTRIBUTING.md
        - LICENSE.md
        - .ls-lint.yml
        - package.json
        - pnpm-lock.yaml
        - tsconfig.json
        - turbo.json
      allow_extra: false
    directories:
      allowed_names:
        - .github
        - apps
        - packages
        - docs
        - scripts
        - infra
        - config
        - tests
      allow_extra: false
    children:
      apps:
        directories:
          naming: kebab-case
        children:
          web-dashboard:
            files:
              naming_patterns:
                "*.js": regex:(next\.config|postcss\.config|tailwind\.config)
                "*.mjs": regex:(eslint\.config)
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
                    "*.jsx": regex:^$
              public:
                files:
                  naming_patterns:
                    "*.png": kebab-case
                    "*.svg": kebab-case
          admin-console:
            files:
              naming_patterns:
                "*.js": regex:(next\.config|postcss\.config|tailwind\.config)
                "*.mjs": regex:(eslint\.config)
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
                    "*.jsx": regex:^$
              public:
                files:
                  naming_patterns:
                    "*.png": kebab-case
                    "*.svg": kebab-case
      packages:
        directories:
          naming: kebab-case
        children:
          core:
            files:
              naming_patterns:
                "*.json": kebab-case
                "*.md": regex:README | kebab-case
            directories:
              naming: kebab-case
            children:
              src:
                files:
                  naming_patterns:
                    "*.ts": kebab-case
                    "*.tsx": PascalCase
                    "*.js": regex:^$
              tests:
                files:
                  naming_patterns:
                    "*.test.ts": kebab-case
          ui-kit:
            files:
              naming_patterns:
                "*.json": kebab-case
                "*.md": regex:README | kebab-case
            directories:
              naming: kebab-case
            children:
              src:
                files:
                  naming_patterns:
                    "*.ts": kebab-case
                    "*.tsx": PascalCase
                    "*.js": regex:^$
              tests:
                files:
                  naming_patterns:
                    "*.test.tsx": kebab-case
      docs:
        files:
          naming_patterns:
            "*.md": regex:(README|AGENTS) | kebab-case
            "*.mdx": kebab-case
            "*.js": regex:^$
            "*.ts": regex:^$
      scripts:
        files:
          naming_patterns:
            "*.sh": kebab-case
            "*.ts": kebab-case
            "*.js": regex:^$
      infra:
        files:
          naming_patterns:
            "*.tf": kebab-case
            "*.yml": kebab-case
            "*.yaml": kebab-case
      config:
        files:
          naming_patterns:
            "*.json": kebab-case
            "*.yml": kebab-case
      tests:
        files:
          naming_patterns:
            "*.test.ts": kebab-case
            "*.spec.ts": kebab-case
      .github:
        children:
          workflows:
            files:
              naming_patterns:
                "*.yml": kebab-case
exclude:
  - ".assura/**"
  - ".ls-lint.yml"
  - "node_modules/**"
  - "apps/*/node_modules"
  - "apps/*/node_modules/**"
  - "packages/*/node_modules/**"
  - "dist/**"
  - "apps/*/dist/**"
  - "packages/*/dist/**"
  - "coverage/**"
  - "apps/*/.next"
  - "apps/*/.next/**"
  - "packages/*/.turbo"
  - "packages/*/.turbo/**"
  - ".turbo"
  - ".turbo/**"
"#,
        r#"
ignore:
  - .assura/**
  - .ls-lint.yml
  - node_modules/**
  - apps/*/node_modules
  - apps/*/node_modules/**
  - packages/*/node_modules/**
  - dist/**
  - apps/*/dist/**
  - packages/*/dist/**
  - coverage/**
  - apps/*/.next
  - apps/*/.next/**
  - packages/*/.turbo
  - packages/*/.turbo/**
  - .turbo
  - .turbo/**
ls:
  .dir: regex:^(\.github|apps|packages|docs|scripts|infra|config|tests|node_modules|dist|coverage)$
  .*: regex:^(README|AGENTS|CONTRIBUTING|LICENSE|package|pnpm-lock|tsconfig|turbo)$
  apps:
    .dir: kebab-case
    web-dashboard:
      .js: regex:(next\.config|postcss\.config|tailwind\.config)
      .mjs: regex:(eslint\.config)
      .json: kebab-case
      src:
        .ts: kebab-case
        .tsx: PascalCase
        .js: regex:^$
        .jsx: regex:^$
      public:
        .png: kebab-case
        .svg: kebab-case
    admin-console:
      .js: regex:(next\.config|postcss\.config|tailwind\.config)
      .mjs: regex:(eslint\.config)
      .json: kebab-case
      src:
        .ts: kebab-case
        .tsx: PascalCase
        .js: regex:^$
        .jsx: regex:^$
      public:
        .png: kebab-case
        .svg: kebab-case
  packages:
    .dir: kebab-case
    core:
      .md: regex:README | kebab-case
      .json: kebab-case
      src:
        .ts: kebab-case
        .tsx: PascalCase
        .js: regex:^$
      tests:
        .test.ts: kebab-case
    ui-kit:
      .md: regex:README | kebab-case
      .json: kebab-case
      src:
        .ts: kebab-case
        .tsx: PascalCase
        .js: regex:^$
      tests:
        .test.tsx: kebab-case
  docs:
    .md: regex:(README|AGENTS) | kebab-case
    .mdx: kebab-case
    .js: regex:^$
    .ts: regex:^$
  scripts:
    .sh: kebab-case
    .ts: kebab-case
    .js: regex:^$
  infra:
    .tf: kebab-case
    .yml: kebab-case
    .yaml: kebab-case
  config:
    .json: kebab-case
    .yml: kebab-case
  tests:
    .test.ts: kebab-case
    .spec.ts: kebab-case
  .github:
    workflows:
      .yml: kebab-case
"#,
    )?;

    for path in [
        ".github/workflows",
        "apps/web-dashboard/src/components",
        "apps/web-dashboard/public",
        "apps/web-dashboard/.next/cache",
        "apps/web-dashboard/node_modules/bad-package",
        "apps/admin-console/src/screens",
        "apps/admin-console/public",
        "apps/admin-console/dist",
        "packages/core/src",
        "packages/core/tests",
        "packages/core/dist",
        "packages/ui-kit/src",
        "packages/ui-kit/tests",
        "packages/ui-kit/.turbo/cache",
        "docs/reference",
        "scripts",
        "infra",
        "config",
        "tests",
        "node_modules/root-package",
        ".turbo/cache",
        "coverage",
    ] {
        fs::create_dir_all(root.join(path)).map_err(|error| format!("create {path}: {error}"))?;
    }

    for file in [
        "README.md",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "LICENSE.md",
        "package.json",
        "pnpm-lock.yaml",
        "tsconfig.json",
        "turbo.json",
        ".github/workflows/ci.yml",
        "apps/web-dashboard/next.config.js",
        "apps/web-dashboard/eslint.config.mjs",
        "apps/web-dashboard/package.json",
        "apps/web-dashboard/src/index.ts",
        "apps/web-dashboard/src/components/DashboardShell.tsx",
        "apps/web-dashboard/public/app-icon.png",
        "apps/admin-console/next.config.js",
        "apps/admin-console/package.json",
        "apps/admin-console/src/index.ts",
        "apps/admin-console/src/screens/AdminHome.tsx",
        "apps/admin-console/public/admin-logo.svg",
        "packages/core/README.md",
        "packages/core/package.json",
        "packages/core/src/index.ts",
        "packages/core/tests/core-api.test.ts",
        "packages/ui-kit/README.md",
        "packages/ui-kit/package.json",
        "packages/ui-kit/src/Button.tsx",
        "packages/ui-kit/src/theme.ts",
        "packages/ui-kit/tests/button-render.test.tsx",
        "docs/README.md",
        "docs/reference/performance-notes.mdx",
        "scripts/bootstrap-dev.sh",
        "scripts/sync-fixtures.ts",
        "infra/main.tf",
        "infra/workers-ci.yml",
        "config/project-policy.json",
        "config/release-policy.yml",
        "tests/app-flow.test.ts",
        "tests/config-loader.spec.ts",
    ] {
        write_file(root.join(file), "")?;
    }

    for generated in [
        "apps/web-dashboard/.next/cache/BAD.js",
        "apps/web-dashboard/node_modules/bad-package/BAD.js",
        "apps/admin-console/dist/BAD.jsx",
        "packages/core/dist/BAD.js",
        "packages/ui-kit/.turbo/cache/BAD.js",
        "node_modules/root-package/BAD.js",
        ".turbo/cache/BAD.js",
        "coverage/BAD.js",
    ] {
        write_file(root.join(generated), "")?;
    }

    Ok(())
}

pub(super) fn create_realistic_rule_heavy_project(root: &Path) -> Result<(), String> {
    let mut rules = String::new();
    for index in 0..36 {
        rules.push_str(&format!("  .kind-{index:02}.ts: kebab-case\n"));
    }
    write_lslint_compatible_configs(
        root,
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
    )?;
    for dir_index in 0..8 {
        let dir = root.join(format!("feature-{dir_index:02}"));
        fs::create_dir(&dir).map_err(|error| format!("create feature dir: {error}"))?;
        for file_index in 0..24 {
            let kind = file_index % 36;
            write_file(
                dir.join(format!(
                    "feature-{dir_index:02}-{file_index:02}.kind-{kind:02}.ts"
                )),
                "",
            )?;
        }
    }
    Ok(())
}

pub(super) fn create_ignored_generated_heavy_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
        r#"
ignore:
  - .assura/**
  - generated/**
  - coverage/**
ls:
  .dir: kebab-case
  .ts: kebab-case
"#,
    )?;
    fs::create_dir(root.join("src")).map_err(|error| format!("create src: {error}"))?;
    write_file(root.join("src/index-file.ts"), "")?;
    for generated_root in ["generated", "coverage"] {
        for dir_index in 0..24 {
            let dir = root.join(format!("{generated_root}/out-{dir_index:02}"));
            fs::create_dir_all(&dir).map_err(|error| format!("create generated dir: {error}"))?;
            for file_index in 0..16 {
                write_file(dir.join(format!("BAD_{file_index:02}.ts")), "")?;
            }
        }
    }
    Ok(())
}
