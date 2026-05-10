//! CLI Commands for Assura
//!
//! Provides command-line interface for validation and migration.

use crate::cli::args::OutputFormat;
use crate::cli::check::{run_structure_check, CheckError, StructureCheckReport};
use crate::cli::{ConfigDiscovery, ExitCode};
use crate::config::config::{Config, DirectoryNode};
use crate::config::loader::ConfigLoader;
use crate::config::parser::ConfigParser;
use crate::ls_compat::MigrationTool;
use crate::validation::{ExecutionContext, ValidationEngine};
use std::path::{Path, PathBuf};

/// Run validation check
pub async fn check_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    format: OutputFormat,
    output: Option<PathBuf>,
    fail_fast: bool,
    _no_parallel: bool,
) -> ExitCode {
    match run_structure_check(path, config, fail_fast) {
        Ok(report) => {
            let rendered = format_structure_report(&report, format);
            if let Some(output) = output {
                if let Err(error) = std::fs::write(&output, rendered) {
                    eprintln!("Error: failed to write report to {:?}: {}", output, error);
                    return ExitCode::RuntimeError;
                }
            } else {
                println!("{}", rendered);
            }

            if report.success {
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
pub async fn init_command(_path: Option<PathBuf>, _force: bool, _no_git_hooks: bool) -> ExitCode {
    println!("Init command not yet implemented");
    ExitCode::Success
}

/// Watch for file changes and validate
pub async fn watch_command(
    _path: Option<PathBuf>,
    _debounce: Option<u64>,
    _no_git: bool,
) -> ExitCode {
    println!("Watch command not yet implemented");
    ExitCode::Success
}

/// CLI command handler
pub struct Cli;

impl Cli {
    /// Run validation check
    pub fn check(config_path: &Path, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading configuration from {:?}...", config_path);

        let config = ConfigParser::parse_file(config_path)?;

        if verbose {
            println!("Loaded {} rules", config.rules.len());
            println!("Loaded {} contexts", config.contexts.len());
        }

        // Detect execution context
        let exec_context = ExecutionContext::from_env();

        if verbose {
            println!("Execution context: {:?}", exec_context);
        }

        let _engine = ValidationEngine::new(config, exec_context);

        // TODO: Walk directory tree and validate files
        println!("Validation complete (files not yet scanned)");

        Ok(())
    }

    /// Migrate from LS-Lint to Assura
    pub fn migrate(
        ls_lint_path: &Path,
        output_path: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Reading LS-Lint config from {:?}...", ls_lint_path);

        let ls_content = std::fs::read_to_string(ls_lint_path)?;

        // Generate migration report
        let report = MigrationTool::generate_report(&ls_content)?;

        println!("\\nMigration Report:");
        println!("  Extension rules: {}", report.extension_rules);
        println!("  Path rules: {}", report.path_rules);
        println!("  Exists rules: {}", report.exists_rules);
        println!("  Ignored patterns: {}", report.ignored_patterns);

        if !report.warnings.is_empty() {
            println!("\\nWarnings:");
            for warning in &report.warnings {
                println!("  - {}", warning);
            }
        }

        // Perform migration
        let assura_yaml = MigrationTool::migrate(&ls_content)?;

        // Write output
        if let Some(output) = output_path {
            std::fs::write(output, assura_yaml)?;
            println!("\\nMigrated config written to {:?}", output);
        } else {
            println!("\\n--- Migrated Configuration ---");
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

fn format_structure_report(report: &StructureCheckReport, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_structure_report_text(report),
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
    }
}

fn format_structure_report_text(report: &StructureCheckReport) -> String {
    let mut output = String::new();
    output.push_str("Assura structure check\n");
    output.push_str("======================\n");
    output.push_str(&format!(
        "Project root: {}\n",
        report.project_root.display()
    ));
    output.push_str(&format!("Config: {}\n", report.config_path.display()));
    output.push_str(&format!(
        "Checked path: {}\n",
        report.checked_path.display()
    ));
    output.push_str(&format!("Files checked: {}\n", report.files_checked));
    output.push_str(&format!("Directories checked: {}\n", report.dirs_checked));
    output.push_str(&format!("Violations: {}\n", report.violation_count()));

    if report.violations.is_empty() {
        output.push_str("\nAll configured structure checks passed.\n");
        return output;
    }

    output.push_str("\nViolations\n");
    output.push_str("----------\n");
    for violation in &report.violations {
        output.push_str(&format!(
            "{} [{}:{}] {}\n",
            violation.path.display(),
            violation.severity,
            violation.rule,
            violation.message
        ));
    }

    output
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

        let result = Cli::migrate(temp_file.path(), None);
        assert!(result.is_ok());
    }
}
