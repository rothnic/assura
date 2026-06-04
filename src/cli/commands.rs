//! CLI Commands for Assura
//!
//! Provides command-line interface for validation and migration.

use crate::cli::args::{AgentTarget, CheckOutputFormat, OutputFormat};
use crate::cli::check::{run_structure_check_with_target_mode, CheckError, CheckTargetMode};
use crate::cli::check_report::format_structure_report;
use crate::cli::init_support::{resolve_project_root, starter_config};
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
pub async fn init_command(path: Option<PathBuf>, force: bool, no_git_hooks: bool) -> ExitCode {
    let project_root = match resolve_project_root(path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Error: {}", error);
            return ExitCode::RuntimeError;
        }
    };

    let assura_dir = project_root.join(".assura");
    let config_path = assura_dir.join("config.yml");
    if config_path.exists() && !force {
        eprintln!(
            "Error: {} already exists. Use --force to overwrite.",
            config_path.display()
        );
        return ExitCode::ConfigurationError;
    }

    if let Err(error) = std::fs::create_dir_all(&assura_dir) {
        eprintln!(
            "Error: failed to create {}: {}",
            assura_dir.display(),
            error
        );
        return ExitCode::RuntimeError;
    }

    if let Err(error) = std::fs::write(&config_path, starter_config()) {
        eprintln!(
            "Error: failed to write {}: {}",
            config_path.display(),
            error
        );
        return ExitCode::RuntimeError;
    }

    println!("Created {}", config_path.display());
    if no_git_hooks {
        println!("Skipped git hook setup because --no-git-hooks was provided.");
    } else {
        println!("Run `assura hooks install` to install optional git hooks.");
    }
    ExitCode::Success
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
pub async fn migrate_command(input: Vec<PathBuf>, output: Option<PathBuf>) -> ExitCode {
    let input = if input.is_empty() {
        vec![PathBuf::from(".ls-lint.yml")]
    } else {
        input
    };
    match Cli::migrate(&input, output.as_deref()) {
        Ok(()) => ExitCode::Success,
        Err(error) => {
            eprintln!("Error: {}", error);
            ExitCode::RuntimeError
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
        ls_lint_paths: &[PathBuf],
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Reading LS-Lint config from {:?}...", ls_lint_paths);

        let ls_contents = ls_lint_paths
            .iter()
            .map(std::fs::read_to_string)
            .collect::<Result<Vec<_>, _>>()?;
        let ls_content_refs = ls_contents.iter().map(String::as_str).collect::<Vec<_>>();

        let migration = convert_ls_lint_documents_to_migration(&ls_content_refs)?;
        let assura_yaml = serde_yaml::to_string(&migration.config)?;
        ConfigLoader::parse_validated(&assura_yaml)?;

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
    use tempfile::NamedTempFile;

    #[test]
    fn test_cli_info() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"
rules:
  react:
    ${{name}}.tsx:
      - constraints: [PascalCase]

policy:
  src/:
    ${{name}}.tsx:
      - apply: react
"#
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

        let result = Cli::migrate(&[temp_file.path().to_path_buf()], None);
        assert!(result.is_ok());
    }
}
