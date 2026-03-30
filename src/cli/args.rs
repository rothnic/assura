use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "assura")]
#[command(about = "Dependency-aware file system validation engine")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Run full validation")]
    Check {
        #[arg(help = "Path to validate (defaults to current directory)")]
        path: Option<PathBuf>,

        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        fail_fast: bool,

        #[arg(long)]
        no_parallel: bool,

        #[arg(long)]
        watch: bool,
    },

    #[command(about = "Show project status")]
    Status {
        #[arg(help = "Path to check (defaults to current directory)")]
        path: Option<PathBuf>,

        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Initialize assura in a project")]
    Init {
        #[arg(help = "Project root directory (defaults to current directory)")]
        path: Option<PathBuf>,

        #[arg(long)]
        force: bool,

        #[arg(long)]
        no_git_hooks: bool,
    },

    #[command(about = "Watch for changes and validate")]
    Watch {
        #[arg(help = "Path to watch (defaults to current directory)")]
        path: Option<PathBuf>,

        #[arg(short, long)]
        debounce: Option<u64>,

        #[arg(long)]
        no_git: bool,
    },

    #[command(about = "Install or manage git hooks")]
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum HookCommands {
    #[command(about = "Install git hooks")]
    Install {
        #[arg(help = "Git hooks directory (defaults to .git/hooks)")]
        path: Option<PathBuf>,

        #[arg(long)]
        force: bool,
    },

    #[command(about = "Remove git hooks")]
    Uninstall {
        #[arg(help = "Git hooks directory (defaults to .git/hooks)")]
        path: Option<PathBuf>,
    },

    #[command(about = "Show installed hooks status")]
    Status,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ExitCode {
    Success = 0,
    ValidationFailed = 1,
    ConfigurationError = 2,
    RuntimeError = 3,
    NoConfigFound = 4,
}
