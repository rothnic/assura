//! CLI Commands for Assura
//!
//! Provides command-line interface for validation and migration.

use crate::cli::args::{AgentTarget, CheckOutputFormat, OutputFormat};
use crate::cli::check::{
    run_markdown_fix, run_structure_check_with_target_mode, CheckError, CheckTargetMode,
};
use crate::cli::check_report::format_structure_report;
use crate::cli::init_support::{
    materialize_recipe_starter_files, materialize_starter, recipe_config, StarterInitError,
};
use crate::cli::MarkdownFixRuleArg;
use crate::cli::{CheckCommandOptions, ConfigDiscovery, ExitCode};
use crate::config::config::{Config, DirectoryNode};
use crate::config::loader::ConfigLoader;
use crate::config::ls_compat::convert_ls_lint_documents_to_migration;
use crate::config::parser::ConfigParser;
use std::path::{Path, PathBuf};

/// Run validation check
pub async fn check_command(options: CheckCommandOptions) -> ExitCode {
    if options.agent != AgentTarget::Generic && options.format != CheckOutputFormat::Agent {
        eprintln!("Error: --agent requires --format agent");
        return ExitCode::ConfigurationError;
    }

    let target_mode = if options.ls_lint_target_semantics {
        CheckTargetMode::LsLint
    } else {
        CheckTargetMode::Recursive
    };
    match run_structure_check_with_target_mode(
        options.path.clone(),
        options.config.clone(),
        options.fail_fast,
        target_mode,
    ) {
        Ok(report) => {
            let rendered = format_structure_report(&report, options.format, &options);
            if let Some(output) = options.output {
                if let Err(error) = std::fs::write(&output, rendered) {
                    eprintln!("Error: failed to write report to {:?}: {}", output, error);
                    return ExitCode::RuntimeError;
                }
            } else {
                println!("{}", rendered);
            }

            if report.success || options.warn {
                ExitCode::Success
            } else {
                ExitCode::ValidationFailed
            }
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            exit_code_for_check_error(&error)
        }
    }
}

/// Show status of configuration
pub async fn status_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    format: OutputFormat,
) -> ExitCode {
    let path = match path {
        Some(path) => path,
        None => match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("Error: failed to read current directory: {}", error);
                return ExitCode::RuntimeError;
            }
        },
    };

    let config_path = match config {
        Some(path) => path,
        None => match ConfigDiscovery::find_config_path(&path) {
            Some(path) if path.exists() => path,
            _ => {
                eprintln!("Error: no .assura/config.yml found for {:?}", path);
                return ExitCode::NoConfigFound;
            }
        },
    };

    let project_root = config_path
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.clone());

    match ConfigLoader::load(&config_path) {
        Ok(config) => {
            let summary = StatusSummary::from_config(project_root, config_path, &config);
            let rendered = match format {
                OutputFormat::Text => summary.format_text(),
                OutputFormat::Json => serde_json::to_string_pretty(&summary).unwrap_or_default(),
                OutputFormat::Yaml => serde_yaml::to_string(&summary).unwrap_or_default(),
                OutputFormat::Advice | OutputFormat::Status => summary.format_text(),
            };
            println!("{}", rendered);
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            ExitCode::ConfigurationError
        }
    }
}

/// Initialize a new Assura configuration
pub async fn init_command(
    path: Option<PathBuf>,
    force: bool,
    no_git_hooks: bool,
    project_intelligence: bool,
    recipes: Vec<crate::cli::args::InitRecipe>,
) -> ExitCode {
    let created = match materialize_starter(path, force, project_intelligence, &recipes) {
        Ok(created) => created,
        Err(error) => {
            eprintln!("Error: {}", error.message());
            return match error {
                StarterInitError::Configuration(_) => ExitCode::ConfigurationError,
                StarterInitError::Runtime(_) => ExitCode::RuntimeError,
            };
        }
    };
    for path in created {
        println!("Created {}", path.display());
    }
    if no_git_hooks {
        println!("Skipped git hook setup because --no-git-hooks was provided.");
    } else {
        println!("Run `assura hooks install` to install optional git hooks.");
    }
    ExitCode::Success
}

/// Merge a first-party recipe into an existing project-owned config.
pub async fn add_recipe_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    recipe: crate::cli::args::InitRecipe,
    dry_run: bool,
    force: bool,
) -> ExitCode {
    match add_recipe(path, config, recipe, dry_run, force) {
        Ok(result) if dry_run => {
            if !result.conflicts.is_empty() {
                println!(
                    "Conflicts that --force would replace: {}",
                    result.conflicts.join(", ")
                );
            }
            println!("{}", result.rendered);
            ExitCode::Success
        }
        Ok(_) => ExitCode::Success,
        Err(StarterInitError::Configuration(message)) => {
            eprintln!("Error: {message}");
            ExitCode::ConfigurationError
        }
        Err(StarterInitError::Runtime(message)) => {
            eprintln!("Error: {message}");
            ExitCode::RuntimeError
        }
    }
}

#[derive(Debug)]
struct RecipeMergeResult {
    rendered: String,
    conflicts: Vec<String>,
}

fn add_recipe(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    recipe: crate::cli::args::InitRecipe,
    dry_run: bool,
    force: bool,
) -> Result<RecipeMergeResult, StarterInitError> {
    let project_root = crate::cli::init_support::resolve_project_root(path)
        .map_err(|error| StarterInitError::Runtime(error.to_string()))?;
    let config_path = config.unwrap_or_else(|| project_root.join(".assura/config.yml"));
    let existing = std::fs::read_to_string(&config_path).map_err(|error| {
        StarterInitError::Runtime(format!("failed to read {}: {error}", config_path.display()))
    })?;
    let mut destination: serde_yaml::Value = serde_yaml::from_str(&existing).map_err(|error| {
        StarterInitError::Configuration(format!(
            "{} is not valid YAML: {error}",
            config_path.display()
        ))
    })?;
    let source: serde_yaml::Value =
        serde_yaml::from_str(recipe_config(recipe)).map_err(|error| {
            StarterInitError::Configuration(format!("built-in recipe is invalid YAML: {error}"))
        })?;
    let mut conflicts = Vec::new();
    merge_recipe_value(
        &mut destination,
        source,
        "",
        force || dry_run,
        &mut conflicts,
    );
    if !conflicts.is_empty() && !force && !dry_run {
        return Err(StarterInitError::Configuration(format!(
            "recipe conflicts at {}. Run with --dry-run to inspect the current merge or --force to use recipe values.",
            conflicts.join(", ")
        )));
    }
    let rendered = serde_yaml::to_string(&destination).map_err(|error| {
        StarterInitError::Runtime(format!("failed to render merged config: {error}"))
    })?;
    ConfigLoader::parse_validated(&rendered).map_err(|error| {
        StarterInitError::Configuration(format!("merged recipe is invalid: {error}"))
    })?;
    if dry_run {
        return Ok(RecipeMergeResult {
            rendered,
            conflicts,
        });
    }

    let temporary = config_path.with_extension("yml.assura-tmp");
    std::fs::write(&temporary, &rendered).map_err(|error| {
        StarterInitError::Runtime(format!("failed to write {}: {error}", temporary.display()))
    })?;
    std::fs::rename(&temporary, &config_path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        StarterInitError::Runtime(format!(
            "failed to replace {}: {error}",
            config_path.display()
        ))
    })?;
    materialize_recipe_starter_files(&project_root, &[recipe])?;
    println!("Updated {}", config_path.display());
    Ok(RecipeMergeResult {
        rendered,
        conflicts,
    })
}

fn merge_recipe_value(
    destination: &mut serde_yaml::Value,
    source: serde_yaml::Value,
    path: &str,
    force: bool,
    conflicts: &mut Vec<String>,
) {
    match (destination, source) {
        (serde_yaml::Value::Mapping(destination), serde_yaml::Value::Mapping(source)) => {
            for (key, value) in source {
                let segment = key.as_str().unwrap_or("<key>");
                let child_path = if path.is_empty() {
                    segment.to_string()
                } else {
                    format!("{path}.{segment}")
                };
                if let Some(existing) = destination.get_mut(&key) {
                    merge_recipe_value(existing, value, &child_path, force, conflicts);
                } else {
                    destination.insert(key, value);
                }
            }
        }
        (serde_yaml::Value::Sequence(destination), serde_yaml::Value::Sequence(source)) => {
            for value in source {
                if !destination.contains(&value) {
                    destination.push(value);
                }
            }
        }
        (destination, source) if *destination == source => {}
        (destination, source) => {
            conflicts.push(path.to_string());
            if force {
                *destination = source;
            }
        }
    }
}

/// Watch for file changes and validate
pub async fn watch_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    debounce: Option<u64>,
    no_git: bool,
) -> ExitCode {
    let debounce = debounce.unwrap_or(300);
    println!(
        "Running one-shot validation for watch mode (debounce: {}ms, git events ignored: {}).",
        debounce, no_git
    );
    check_command(CheckCommandOptions {
        path,
        config,
        format: crate::cli::args::CheckOutputFormat::Text,
        agent: crate::cli::args::AgentTarget::Generic,
        min_severity: None,
        max_issues: None,
        output: None,
        fail_fast: false,
        warn: false,
        ls_lint_target_semantics: false,
    })
    .await
}

/// Migrate an LS-Lint configuration to Assura structure config.
pub async fn migrate_command(
    input: Vec<PathBuf>,
    from: crate::cli::args::MigrationSource,
    output: Option<PathBuf>,
) -> ExitCode {
    let input = if input.is_empty() {
        vec![PathBuf::from(".ls-lint.yml")]
    } else {
        input
    };
    match Cli::migrate(&input, from, output.as_deref()) {
        Ok(()) => ExitCode::Success,
        Err(error) => {
            eprintln!("Error: {}", error);
            ExitCode::RuntimeError
        }
    }
}

/// Apply safe Markdown fixes.
pub async fn fix_markdown_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    rule: MarkdownFixRuleArg,
    dry_run: bool,
    apply: bool,
    format: OutputFormat,
) -> ExitCode {
    if dry_run && apply {
        eprintln!("Error: --dry-run and --apply cannot be used together");
        return ExitCode::RuntimeError;
    }

    let rule = rule.into();
    let dry_run = !apply;

    match run_markdown_fix(path, config, rule, dry_run) {
        Ok(report) => {
            println!("{}", format_markdown_fix_report(&report, format));
            if report.failures.is_empty() {
                ExitCode::Success
            } else {
                ExitCode::RuntimeError
            }
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            exit_code_for_check_error(&error)
        }
    }
}

fn format_markdown_fix_report(
    report: &crate::cli::check::MarkdownFixReport,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => {
            if report.dry_run {
                format!(
                    "Checked {} Markdown file(s); would change {} file(s); would apply {} fix(es). Run again with --apply to write these fixes.",
                    report.files_checked, report.files_would_change, report.fixes_would_apply
                )
            } else {
                format!(
                    "Checked {} Markdown file(s); changed {} file(s); applied {} fix(es); failed {} operation(s).",
                    report.files_checked,
                    report.files_changed,
                    report.fixes_applied,
                    report.failures.len()
                )
            }
        }
    }
}

/// Show configuration information.
pub async fn info_command(path: Option<PathBuf>, config: Option<PathBuf>) -> ExitCode {
    let config_path = match path.or(config) {
        Some(path) => path,
        None => {
            let cwd = match std::env::current_dir() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("Error: failed to read current directory: {}", error);
                    return ExitCode::RuntimeError;
                }
            };
            match ConfigDiscovery::find_config_path(&cwd) {
                Some(path) if path.exists() => path,
                _ => {
                    eprintln!("Error: no .assura/config.yml found for {:?}", cwd);
                    return ExitCode::NoConfigFound;
                }
            }
        }
    };

    match ConfigLoader::load(&config_path) {
        Ok(config) => {
            println!("Assura configuration info");
            println!("=========================");
            println!("Config: {}", config_path.display());
            println!("Structure roots: {}", config.structure.len());
            println!("Top-level patterns: {}", config.patterns.len());
            println!("Exclusions: {}", config.exclude.len());
            println!(
                "LS-Lint compatibility section: {}",
                if config.ls.is_some() { "yes" } else { "no" }
            );
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            ExitCode::ConfigurationError
        }
    }
}

/// CLI command handler
pub struct Cli;

impl Cli {
    /// Migrate from LS-Lint to Assura
    pub fn migrate(
        input_paths: &[PathBuf],
        source: crate::cli::args::MigrationSource,
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contents = input_paths
            .iter()
            .map(std::fs::read_to_string)
            .collect::<Result<Vec<_>, _>>()?;
        let source = resolve_migration_source(source, &contents)?;
        println!("Reading {:?} config from {:?}...", source, input_paths);

        let assura_yaml = match source {
            crate::cli::args::MigrationSource::LsLint => {
                let refs = contents.iter().map(String::as_str).collect::<Vec<_>>();
                let migration = convert_ls_lint_documents_to_migration(&refs)?;
                println!("\nMigration Report:");
                println!("  Extension rules: {}", migration.report.extension_rules);
                println!("  Path rules: {}", migration.report.path_rules);
                println!("  Exists rules: {}", migration.report.exists_rules);
                println!("  Ignored patterns: {}", migration.report.ignored_patterns);
                if !migration.report.warnings.is_empty() {
                    println!("\nWarnings:");
                    for warning in &migration.report.warnings {
                        println!("  - {}", warning);
                    }
                }
                serde_yaml::to_string(&migration.config)?
            }
            crate::cli::args::MigrationSource::AssuraV1 => {
                if contents.len() != 1 {
                    return Err("Assura v1 migration accepts exactly one input file".into());
                }
                let config = migrate_assura_v1(&contents[0])?;
                println!("\nMigration Report:");
                println!("  Legacy Assura config normalized into the current schema");
                serde_yaml::to_string(&config)?
            }
            crate::cli::args::MigrationSource::Auto => unreachable!(),
        };
        ConfigLoader::parse_validated(&assura_yaml)?;

        // Write output
        if let Some(output) = output_path {
            if let Some(parent) = output.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(output, assura_yaml)?;
            println!("\nMigrated config written to {:?}", output);
        } else {
            println!("\n--- Migrated Configuration ---");
            println!("{}", assura_yaml);
        }

        Ok(())
    }

    /// Show configuration information
    pub fn info(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConfigParser::parse_file(config_path)?;

        println!("Assura Configuration Info");
        println!("=========================");
        println!("Rules: {}", config.rules.len());
        println!("Contexts: {}", config.contexts.len());
        println!("Policy entries: {}", config.policy.entries.len());

        if !config.rules.is_empty() {
            println!("\\nDefined Rules:");
            for name in config.rules.keys() {
                println!("  - {}", name);
            }
        }

        if !config.contexts.is_empty() {
            println!("\\nDefined Contexts:");
            for (name, ctx) in &config.contexts {
                print!("  - {}", name);
                if let Some(ref hook) = ctx.hook {
                    print!(" (hook: {})", hook);
                }
                println!();
            }
        }

        Ok(())
    }
}

fn migrate_assura_v1(content: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut value: serde_yaml::Value = serde_yaml::from_str(content)?;
    let root = value
        .as_mapping_mut()
        .ok_or("legacy Assura config must be a YAML mapping")?;

    if let Some(serde_yaml::Value::Mapping(rules)) =
        root.get_mut(serde_yaml::Value::String("rules".into()))
    {
        let legacy = std::mem::take(rules);
        for (key, value) in legacy {
            let key = match key {
                serde_yaml::Value::String(name) => serde_yaml::Value::String(
                    legacy_rule_name(&name).unwrap_or(name.as_str()).to_string(),
                ),
                key => key,
            };
            rules.insert(key, value);
        }
    }
    rewrite_legacy_rule_references(&mut value);
    let normalized = serde_yaml::to_string(&value)?;
    Ok(ConfigLoader::parse_validated(&normalized)?)
}

fn legacy_rule_name(value: &str) -> Option<&str> {
    let name = value.strip_prefix('@')?;
    (!name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_')))
    .then_some(name)
}

fn rewrite_legacy_rule_references(value: &mut serde_yaml::Value) {
    match value {
        serde_yaml::Value::String(text) => {
            if let Some(name) = legacy_rule_name(text) {
                *text = format!("${name}");
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for value in values {
                rewrite_legacy_rule_references(value);
            }
        }
        serde_yaml::Value::Mapping(mapping) => {
            for value in mapping.values_mut() {
                rewrite_legacy_rule_references(value);
            }
        }
        _ => {}
    }
}

fn resolve_migration_source(
    requested: crate::cli::args::MigrationSource,
    contents: &[String],
) -> Result<crate::cli::args::MigrationSource, Box<dyn std::error::Error>> {
    if requested != crate::cli::args::MigrationSource::Auto {
        return Ok(requested);
    }
    let mut detected = None;
    for content in contents {
        let value: serde_yaml::Value = serde_yaml::from_str(content)?;
        let mapping = value
            .as_mapping()
            .ok_or("migration input must be a YAML mapping")?;
        let current = if mapping.contains_key(serde_yaml::Value::String("ls".into())) {
            crate::cli::args::MigrationSource::LsLint
        } else if mapping.contains_key(serde_yaml::Value::String("structure".into()))
            || mapping.contains_key(serde_yaml::Value::String("rules".into()))
        {
            crate::cli::args::MigrationSource::AssuraV1
        } else {
            return Err(
                "cannot detect migration input; pass --from ls-lint or --from assura-v1".into(),
            );
        };
        if detected.is_some_and(|source| source != current) {
            return Err("all migration inputs must use the same source grammar".into());
        }
        detected = Some(current);
    }
    detected.ok_or_else(|| "at least one migration input is required".into())
}

fn exit_code_for_check_error(error: &CheckError) -> ExitCode {
    match error {
        CheckError::NoConfig(_) => ExitCode::NoConfigFound,
        CheckError::Config(_) => ExitCode::ConfigurationError,
        _ => ExitCode::RuntimeError,
    }
}

#[derive(Debug, serde::Serialize)]
struct StatusSummary {
    project_root: PathBuf,
    config_path: PathBuf,
    configured_directories: usize,
    configured_file_rules: usize,
    configured_markdown_rules: usize,
    exclusions: Vec<String>,
}

impl StatusSummary {
    fn from_config(project_root: PathBuf, config_path: PathBuf, config: &Config) -> Self {
        let mut summary = Self {
            project_root,
            config_path,
            configured_directories: 0,
            configured_file_rules: 0,
            configured_markdown_rules: 0,
            exclusions: config.exclude.clone(),
        };

        for node in config.structure.values() {
            summary.count_node(node);
        }

        summary
    }

    fn count_node(&mut self, node: &DirectoryNode) {
        self.configured_directories += 1;
        if node.files.is_some() {
            self.configured_file_rules += 1;
        }
        if node.markdown.is_some() {
            self.configured_markdown_rules += 1;
        }
        if let Some(children) = &node.children {
            for child in children.values() {
                self.count_node(child);
            }
        }
    }

    fn format_text(&self) -> String {
        format!(
            "\
Assura project status
=====================
Project root: {}
Config: {}
Configured directories: {}
File rule bundles: {}
Markdown rule bundles: {}
Exclusions: {}
",
            self.project_root.display(),
            self.config_path.display(),
            self.configured_directories,
            self.configured_file_rules,
            self.configured_markdown_rules,
            if self.exclusions.is_empty() {
                "none".to_string()
            } else {
                self.exclusions.join(", ")
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_cli_info() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(
                r#"
rules:
  react:
    "{name}.tsx":
      - constraints: [PascalCase]

policy:
  src/:
    "{name}.tsx":
      - apply: react
"#
                .as_bytes(),
            )
            .unwrap();

        let result = Cli::info(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_migrate() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"
ls:
  .rs: snake_case
  .tsx: PascalCase
"#
        )
        .unwrap();

        let result = Cli::migrate(
            &[temp_file.path().to_path_buf()],
            crate::cli::args::MigrationSource::Auto,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn migration_source_auto_detects_legacy_assura() {
        let source = resolve_migration_source(
            crate::cli::args::MigrationSource::Auto,
            &["structure:\n  ./:\n    extra: true\n".into()],
        )
        .unwrap();
        assert_eq!(source, crate::cli::args::MigrationSource::AssuraV1);
    }

    #[test]
    fn assura_v1_migration_normalizes_legacy_rule_sigil() {
        let config = migrate_assura_v1(
            r#"rules:
  "@source-file":
    max_lines: 500
structure:
  ./**/*.ts: "@source-file"
"#,
        )
        .unwrap();

        let rendered = serde_yaml::to_string(&config).unwrap();
        assert!(!rendered.contains("@source-file"));
        assert!(rendered.contains("500"));
        ConfigLoader::parse_validated(&rendered).unwrap();
    }

    #[test]
    fn add_recipe_previews_and_writes_project_owned_yaml() {
        let root = tempdir().unwrap();
        let assura = root.path().join(".assura");
        std::fs::create_dir_all(&assura).unwrap();
        let config_path = assura.join("config.yml");
        std::fs::write(&config_path, "structure:\n  package.json: exists:0-1\n").unwrap();

        let preview = add_recipe(
            Some(root.path().to_path_buf()),
            None,
            crate::cli::args::InitRecipe::AgenticCore,
            true,
            false,
        )
        .unwrap();
        assert!(preview.rendered.contains("agent-entrypoint"));
        assert!(!std::fs::read_to_string(&config_path)
            .unwrap()
            .contains("agent-entrypoint"));

        add_recipe(
            Some(root.path().to_path_buf()),
            None,
            crate::cli::args::InitRecipe::AgenticCore,
            false,
            false,
        )
        .unwrap();
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(written.contains("agent-entrypoint"));
        ConfigLoader::parse_validated(&written).unwrap();
        assert!(root.path().join("AGENTS.md").exists());
        assert!(root.path().join("README.md").exists());
        assert!(root.path().join("docs/agent-guidance.md").exists());
    }

    #[test]
    fn add_recipe_reports_conflicts_before_writing() {
        let root = tempdir().unwrap();
        let assura = root.path().join(".assura");
        std::fs::create_dir_all(&assura).unwrap();
        let config_path = assura.join("config.yml");
        let original = "rules:\n  agent-entrypoint:\n    max_lines: 999\nstructure: {}\n";
        std::fs::write(&config_path, original).unwrap();

        let error = add_recipe(
            Some(root.path().to_path_buf()),
            None,
            crate::cli::args::InitRecipe::AgenticCore,
            false,
            false,
        )
        .unwrap_err();
        assert!(error.message().contains("rules.agent-entrypoint.max_lines"));
        assert_eq!(std::fs::read_to_string(config_path).unwrap(), original);
    }
}
