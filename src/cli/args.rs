//! Command-line argument definitions for the Assura CLI.
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "assura")]
#[command(about = "Structure-first repository validation CLI")]
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
        format: CheckOutputFormat,

        #[arg(
            long,
            value_enum,
            default_value = "generic",
            help = "Delivery adapter for --format agent"
        )]
        agent: AgentTarget,

        #[arg(
            long,
            value_parser = ["low", "medium", "high", "critical"],
            help = "Only show feedback items for this severity or higher"
        )]
        min_severity: Option<String>,

        #[arg(long, help = "Maximum feedback items to show")]
        max_issues: Option<usize>,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        fail_fast: bool,

        #[arg(long, help = "Report violations but exit successfully")]
        warn: bool,

        #[arg(long)]
        no_parallel: bool,

        #[arg(
            long,
            help = "Match LS-Lint path-argument behavior by validating only the explicit target path"
        )]
        ls_lint_target_semantics: bool,

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

    #[command(about = "Migrate an LS-Lint configuration to Assura")]
    Migrate {
        #[arg(help = "LS-Lint configuration path(s); defaults to .ls-lint.yml")]
        input: Vec<PathBuf>,

        #[arg(short, long, help = "Output path for generated Assura config")]
        output: Option<PathBuf>,
    },

    #[command(about = "Show Assura configuration information")]
    Info {
        #[arg(help = "Assura configuration path (defaults to discovered config)")]
        path: Option<PathBuf>,
    },

    #[command(about = "Emit Assura versus LS-Lint performance comparison data")]
    PerformanceReport {
        #[arg(
            short,
            long,
            help = "Output path for the current run report (defaults to stdout)"
        )]
        output: Option<PathBuf>,

        #[arg(long, help = "Append JSONL result rows to this history file")]
        history: Option<PathBuf>,

        #[arg(
            long,
            help = "Copy current JSON and JSONL history into this website public data directory"
        )]
        website_dir: Option<PathBuf>,

        #[arg(
            long,
            default_value_t = 3,
            help = "Measured iterations per tool and fixture"
        )]
        iterations: usize,

        #[arg(long, default_value = "stable-baseline-v1")]
        baseline_id: String,

        #[arg(long, default_value = "json", value_enum)]
        format: PerformanceReportFormat,

        #[arg(long, default_value = "@ls-lint/ls-lint@2.3.0")]
        ls_lint_package: String,

        #[arg(
            long,
            help = "Include pinned external Git fixtures; may clone large repositories"
        )]
        include_external_fixtures: bool,
    },

    #[command(about = "Install or manage git hooks")]
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },

    #[command(about = "Plan quality gates for changed files")]
    Quality {
        #[command(subcommand)]
        command: QualityCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum QualityCommands {
    #[command(about = "Plan required quality gates for changed files")]
    Plan {
        #[arg(help = "Project root directory (defaults to current directory)")]
        path: Option<PathBuf>,

        #[arg(long, help = "Read changed paths from a file, or '-' for stdin")]
        files_from: Option<String>,

        #[arg(long, help = "Base git revision for diff-based planning")]
        base: Option<String>,

        #[arg(long, help = "Head git revision for diff-based planning")]
        head: Option<String>,

        #[arg(long, value_enum, default_value = "pr")]
        phase: QualityPhase,

        #[arg(short, long, value_enum, default_value = "text")]
        format: QualityPlanFormat,
    },
}

#[derive(Subcommand, Debug)]
pub enum HookCommands {
    #[command(about = "Install git hooks")]
    Install {
        #[arg(help = "Project root directory (defaults to current directory)")]
        path: Option<PathBuf>,

        #[arg(long)]
        force: bool,
    },

    #[command(about = "Remove git hooks")]
    Uninstall {
        #[arg(help = "Project root directory (defaults to current directory)")]
        path: Option<PathBuf>,
    },

    #[command(about = "Show installed hooks status")]
    Status {
        #[arg(help = "Project root directory (defaults to discovered project root)")]
        path: Option<PathBuf>,
    },

    #[command(about = "Verify installed hooks are managed and runnable")]
    Verify {
        #[arg(help = "Project root directory (defaults to discovered project root)")]
        path: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Advice,
    Status,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum CheckOutputFormat {
    Text,
    Json,
    Yaml,
    Advice,
    Status,
    Agent,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentTarget {
    Generic,
    Codex,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum PerformanceReportFormat {
    Json,
    Jsonl,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum QualityPhase {
    Frequent,
    PrePush,
    Pr,
    Merge,
    Release,
    Scheduled,
}

impl QualityPhase {
    /// Stable config key for this workflow phase.
    pub fn as_config_key(self) -> &'static str {
        match self {
            Self::Frequent => "frequent",
            Self::PrePush => "pre_push",
            Self::Pr => "pr",
            Self::Merge => "merge",
            Self::Release => "release",
            Self::Scheduled => "scheduled",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum QualityPlanFormat {
    Text,
    Json,
    Github,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ExitCode {
    Success = 0,
    ValidationFailed = 1,
    ConfigurationError = 2,
    RuntimeError = 3,
    NoConfigFound = 4,
}
