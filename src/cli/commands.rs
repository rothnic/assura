//! CLI Commands for Assura
//!
//! Provides command-line interface for validation and migration.

use crate::cli::args::OutputFormat;
use crate::cli::ExitCode;
use crate::config::parser::ConfigParser;
use crate::validation::{ValidationEngine, ExecutionContext, ViolationLevel};
use crate::ls_compat::MigrationTool;
use std::path::{Path, PathBuf};

/// Run validation check
pub async fn check_command(
    _path: Option<PathBuf>,
    _format: OutputFormat,
    _output: Option<PathBuf>,
    _fail_fast: bool,
    _no_parallel: bool,
) -> ExitCode {
    println!("Check command not yet implemented");
    ExitCode::Success
}

/// Show status of configuration
pub async fn status_command(_path: Option<PathBuf>, _format: OutputFormat) -> ExitCode {
    println!("Status command not yet implemented");
    ExitCode::Success
}

/// Initialize a new Assura configuration
pub async fn init_command(
    _path: Option<PathBuf>,
    _force: bool,
    _no_git_hooks: bool,
) -> ExitCode {
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
        
        let engine = ValidationEngine::new(config, exec_context);
        
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
            for (name, _) in &config.rules {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cli_info() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, r#"
rules:
  react:
    ${{name}}.tsx:
      - constraints: [PascalCase]

policy:
  src/:
    ${{name}}.tsx:
      - apply: react
"#).unwrap();
        
        let result = Cli::info(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_migrate() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, r#"
ls:
  .rs: snake_case
  .tsx: PascalCase
"#).unwrap();
        
        let result = Cli::migrate(temp_file.path(), None);
        assert!(result.is_ok());
    }
}
