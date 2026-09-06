//! Full multi-command CLI entrypoint used by `assura-full`.

use clap::Parser;
use std::ffi::OsString;
use tracing::{error, info};

use super::{
    agent_command, cache_command, check_command, content_command, daemon_command, editor_command,
    explain_command, fix_markdown_command, info_command, init_command, migrate_command,
    performance_report_command, project_review_command, quality_plan_command, status_command,
    watch_command, CacheCommands, CheckCommandOptions, Cli, Commands, ConfigCommands, ExitCode,
    FixCommands, HookCommands, PerformanceReportCommandOptions, QualityCommands,
};

/// Run the complete Clap/Tokio-powered CLI for non-check commands and fallbacks.
pub fn run_full_cli_from_env() -> i32 {
    run_full_cli_from_args(full_cli_args_from_env())
}

fn full_cli_args_from_env() -> Vec<OsString> {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    if let Ok(bin_name) = std::env::var("ASSURA_CLI_BIN_NAME") {
        if let Some(first) = args.first_mut() {
            *first = OsString::from(bin_name);
        }
    }
    args
}

fn run_full_cli_from_args<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to initialize Tokio runtime");
    let cli = Cli::parse_from(args);
    let exit_code = runtime.block_on(run_full_cli(cli));
    exit_code as i32
}

async fn run_full_cli(cli: Cli) -> ExitCode {
    // Initialize tracing
    tracing_subscriber::fmt::init();

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
    let verbose = cli.verbose;

    match cli.command {
        Commands::Check {
            path,
            format,
            agent,
            min_severity,
            max_issues,
            output,
            fail_fast,
            warn,
            no_parallel: _,
            ls_lint_target_semantics,
            watch: _,
        } => {
            check_command(CheckCommandOptions {
                path,
                config: config_path,
                format,
                agent,
                min_severity,
                max_issues,
                output,
                fail_fast,
                warn,
                ls_lint_target_semantics,
            })
            .await
        }
        Commands::Status { path, format } => status_command(path, config_path, format).await,
        Commands::Doctor { path, format } => super::doctor_command(path, config_path, format).await,
        Commands::Review { path, format, base } => {
            project_review_command(path, config_path, format, base, verbose).await
        }
        Commands::Cache { command } => match command {
            CacheCommands::Status {
                path,
                cache_dir,
                format,
            } => cache_command(path, cache_dir, format, false),
            CacheCommands::Clean {
                path,
                cache_dir,
                format,
            } => cache_command(path, cache_dir, format, true),
        },
        Commands::Explain { path, format } => explain_command(path, config_path, format).await,
        Commands::Init {
            path,
            project_intelligence,
            force,
            no_git_hooks,
            recipe,
            recipe_file,
        } => {
            init_command(
                path,
                force,
                no_git_hooks,
                project_intelligence,
                recipe,
                recipe_file,
            )
            .await
        }
        Commands::Config { command } => match command {
            ConfigCommands::AddRecipe {
                recipe,
                path,
                dry_run,
                force,
            } => super::add_recipe_command(path, config_path, recipe, dry_run, force).await,
        },
        Commands::Watch {
            path,
            debounce,
            format,
            no_git,
        } => watch_command(path, config_path, debounce, format, no_git).await,
        Commands::Migrate {
            input,
            from,
            output,
        } => migrate_command(input, from, output).await,
        Commands::Fix { command } => match command {
            FixCommands::Markdown {
                path,
                rule,
                dry_run,
                apply,
                format,
            } => fix_markdown_command(path, config_path, rule, dry_run, apply, format).await,
        },
        Commands::Agent { command } => agent_command(command, config_path).await,
        Commands::Editor { command } => editor_command(command, config_path).await,
        Commands::Content { command } => content_command(command, config_path).await,
        Commands::Daemon { command } => daemon_command(command, config_path).await,
        Commands::Info { path } => info_command(path, config_path).await,
        Commands::PerformanceReport {
            output,
            history,
            website_dir,
            iterations,
            baseline_id,
            format,
            ls_lint_package,
            suite,
            include_external_fixtures,
        } => {
            performance_report_command(PerformanceReportCommandOptions {
                output,
                history,
                website_dir,
                iterations,
                baseline_id,
                format,
                ls_lint_package,
                suite,
                include_external_fixtures,
            })
            .await
        }
        Commands::Hooks { command } => match command {
            HookCommands::Install { path, force } => handle_hooks_install(path, force).await,
            HookCommands::Uninstall { path } => handle_hooks_uninstall(path).await,
            HookCommands::Status { path } => handle_hooks_status(path).await,
            HookCommands::Verify { path } => handle_hooks_verify(path).await,
        },
        Commands::Quality { command } => match command {
            QualityCommands::Plan {
                path,
                files_from,
                base,
                head,
                phase,
                format,
            } => {
                quality_plan_command(super::QualityPlanCommandOptions {
                    path,
                    config: config_path,
                    files_from,
                    base,
                    head,
                    phase,
                    format,
                })
                .await
            }
        },
    }
}

async fn handle_hooks_install(path: Option<std::path::PathBuf>, force: bool) -> ExitCode {
    use super::hooks::GitHooksManager;

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
    use super::hooks::GitHooksManager;

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

async fn handle_hooks_status(path: Option<std::path::PathBuf>) -> ExitCode {
    use super::config::ConfigDiscovery;
    use super::hooks::GitHooksManager;

    let project_root = match path {
        Some(path) => path,
        None => match ConfigDiscovery::find_project_root(".") {
            Some(root) => root,
            None => {
                eprintln!("Error: Could not find project root");
                return ExitCode::NoConfigFound;
            }
        },
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

async fn handle_hooks_verify(path: Option<std::path::PathBuf>) -> ExitCode {
    use super::config::ConfigDiscovery;
    use super::hooks::GitHooksManager;

    let project_root = match path {
        Some(path) => path,
        None => match ConfigDiscovery::find_project_root(".") {
            Some(root) => root,
            None => {
                eprintln!("Error: Could not find project root");
                return ExitCode::NoConfigFound;
            }
        },
    };

    match GitHooksManager::new(&project_root) {
        Ok(manager) => {
            let statuses = manager.all_status();
            println!("Git hooks verification:");
            for status in &statuses {
                println!("{}", status.display());
            }

            let failure_count = statuses.iter().filter(|status| !status.is_ready()).count();
            if failure_count == 0 {
                println!("All Assura hooks are installed, managed, and runnable.");
                ExitCode::Success
            } else {
                eprintln!("Error: {failure_count} hook(s) are not ready.");
                ExitCode::ValidationFailed
            }
        }
        Err(e) => {
            error!("Git repository not found: {}", e);
            eprintln!("Error: {}", e);
            ExitCode::ConfigurationError
        }
    }
}
