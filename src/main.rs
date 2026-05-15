//! Assura command-line binary entrypoint.
use std::process;

use clap::Parser;
use tracing::{error, info};

use assura::cli::{
    check_command, info_command, init_command, migrate_command, performance_report_command,
    status_command, watch_command, Cli, Commands, ExitCode, HookCommands,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Set log level based on verbosity
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if cli.quiet {
        std::env::set_var("RUST_LOG", "error");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    info!("Starting Assura CLI");
    let config_path = cli.config.clone();

    let exit_code = match cli.command {
        Commands::Check {
            path,
            format,
            output,
            fail_fast,
            no_parallel,
            watch: _,
        } => check_command(path, config_path, format, output, fail_fast, no_parallel).await,
        Commands::Status { path, format } => status_command(path, config_path, format).await,
        Commands::Init {
            path,
            force,
            no_git_hooks,
        } => init_command(path, force, no_git_hooks).await,
        Commands::Watch {
            path,
            debounce,
            no_git,
        } => watch_command(path, config_path, debounce, no_git).await,
        Commands::Migrate { input, output } => migrate_command(input, output).await,
        Commands::Info { path } => info_command(path, config_path).await,
        Commands::PerformanceReport {
            output,
            history,
            website_dir,
            iterations,
            baseline_id,
            format,
            ls_lint_package,
        } => {
            performance_report_command(
                output,
                history,
                website_dir,
                iterations,
                baseline_id,
                format,
                ls_lint_package,
            )
            .await
        }
        Commands::Hooks { command } => match command {
            HookCommands::Install { path, force } => handle_hooks_install(path, force).await,
            HookCommands::Uninstall { path } => handle_hooks_uninstall(path).await,
            HookCommands::Status => handle_hooks_status().await,
        },
    };

    process::exit(exit_code as i32);
}

async fn handle_hooks_install(path: Option<std::path::PathBuf>, force: bool) -> ExitCode {
    use assura::cli::hooks::GitHooksManager;

    let project_root = path.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });

    match GitHooksManager::new(&project_root) {
        Ok(manager) => match manager.install_all(force) {
            Ok(installed) => {
                if installed.is_empty() {
                    println!("No hooks installed (already exist). Use --force to overwrite.");
                } else {
                    println!("Installed hooks:");
                    for hook in installed {
                        println!("  ✓ {}", hook.as_str());
                    }
                }
                ExitCode::Success
            }
            Err(e) => {
                error!("Failed to install hooks: {}", e);
                eprintln!("Error: {}", e);
                ExitCode::RuntimeError
            }
        },
        Err(e) => {
            error!("Git repository not found: {}", e);
            eprintln!("Error: {}", e);
            ExitCode::ConfigurationError
        }
    }
}

async fn handle_hooks_uninstall(path: Option<std::path::PathBuf>) -> ExitCode {
    use assura::cli::hooks::GitHooksManager;

    let project_root = path.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });

    match GitHooksManager::new(&project_root) {
        Ok(manager) => match manager.uninstall_all() {
            Ok(uninstalled) => {
                if uninstalled.is_empty() {
                    println!("No hooks to uninstall.");
                } else {
                    println!("Uninstalled hooks:");
                    for hook in uninstalled {
                        println!("  ✓ {}", hook.as_str());
                    }
                }
                ExitCode::Success
            }
            Err(e) => {
                error!("Failed to uninstall hooks: {}", e);
                eprintln!("Error: {}", e);
                ExitCode::RuntimeError
            }
        },
        Err(e) => {
            error!("Git repository not found: {}", e);
            eprintln!("Error: {}", e);
            ExitCode::ConfigurationError
        }
    }
}

async fn handle_hooks_status() -> ExitCode {
    use assura::cli::config::ConfigDiscovery;
    use assura::cli::hooks::GitHooksManager;

    let project_root = match ConfigDiscovery::find_project_root(".") {
        Some(root) => root,
        None => {
            eprintln!("Error: Could not find project root");
            return ExitCode::NoConfigFound;
        }
    };

    match GitHooksManager::new(&project_root) {
        Ok(manager) => {
            println!("Git hooks status:");
            for status in manager.all_status() {
                println!("{}", status.display());
            }
            ExitCode::Success
        }
        Err(e) => {
            error!("Git repository not found: {}", e);
            eprintln!("Error: {}", e);
            ExitCode::ConfigurationError
        }
    }
}
