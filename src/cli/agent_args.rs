//! Agent-facing project-intelligence command arguments.

use super::args::OutputFormat;
use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    #[command(about = "Bootstrap a broad agent-ready project baseline")]
    Onboard {
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "auto")]
        agent: AgentOnboardingTarget,
        #[arg(long, value_enum, default_value = "none")]
        content_template: AgentContentTemplate,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Report shared project-intelligence agent context")]
    Context {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Report project-intelligence diagnostics through the shared agent envelope")]
    Diagnostics {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Build one bounded project-intelligence context pack")]
    ContextPack {
        path: Option<PathBuf>,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Show one modeled content instance")]
    Show {
        collection: String,
        id: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Search modeled content facts by keyword")]
    Search {
        query: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Report relationship edges with missing targets")]
    MissingRelations {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Expand bounded graph context around one content instance")]
    Expand {
        collection: String,
        id: String,
        path: Option<PathBuf>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Preview safe deterministic project-intelligence fixes")]
    SafeFixes {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Emit bounded event-aware nudges for local coding agents")]
    Nudge {
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "session-start")]
        event: AgentNudgeEvent,
        #[arg(long = "changed", help = "Changed path relevant to this agent event")]
        changed_paths: Vec<PathBuf>,
        #[arg(long, value_enum, default_value = "generic")]
        agent: AgentNudgeTarget,
        #[arg(
            long,
            value_parser = ["low", "medium", "high", "critical"],
            default_value = "medium",
            help = "Only include finding nudges for this severity or higher"
        )]
        min_severity: String,
        #[arg(long, default_value_t = 5)]
        max_issues: usize,
        #[arg(long, default_value_t = 20)]
        reference_limit: usize,
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Install or manage local host-agent integration bundles")]
    Integration {
        #[command(subcommand)]
        command: AgentIntegrationCommands,
    },

    #[command(about = "Run a persistent JSON-line project-intelligence query session")]
    Session { path: Option<PathBuf> },
}

/// Lifecycle commands for generated host-agent integration bundles.
#[derive(Subcommand, Debug)]
pub enum AgentIntegrationCommands {
    /// Generate reviewable wrapper files for one host agent.
    Install(AgentIntegrationLifecycleArgs),
    /// Regenerate an existing wrapper bundle.
    Update(AgentIntegrationLifecycleArgs),
    /// Remove an Assura-managed wrapper bundle.
    Remove(AgentIntegrationLifecycleArgs),
    /// Report whether an Assura-managed wrapper bundle is present.
    Status(AgentIntegrationStatusArgs),
    /// Diagnose config, daemon, and wrapper-bundle readiness.
    Doctor(AgentIntegrationStatusArgs),
}

/// Shared install/update/remove arguments.
#[derive(clap::Args, Debug)]
pub struct AgentIntegrationLifecycleArgs {
    /// Host agent integration to manage.
    #[arg(value_enum)]
    pub agent: AgentIntegrationTarget,

    /// Project root directory.
    pub path: Option<PathBuf>,

    /// Preview file actions without writing.
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite managed files even when they already exist.
    #[arg(long)]
    pub force: bool,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

/// Shared status/doctor arguments.
#[derive(clap::Args, Debug)]
pub struct AgentIntegrationStatusArgs {
    /// Host agent integration to inspect.
    #[arg(value_enum)]
    pub agent: AgentIntegrationTarget,

    /// Project root directory.
    pub path: Option<PathBuf>,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

/// Supported host-agent integration targets.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentIntegrationTarget {
    /// Codex hook or command-wrapper bundle.
    Codex,
    /// OpenCode plugin or hook-wrapper bundle.
    Opencode,
    /// Claude Code hook-wrapper bundle.
    Claude,
    /// Pi agent extension or hook-wrapper bundle.
    Pi,
}

/// Host-agent target for the first-run onboarding flow.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentOnboardingTarget {
    /// Detect a known local host, otherwise use the generic shell profile.
    Auto,
    /// Vendor-neutral shell and AGENTS.md guidance only.
    Generic,
    /// Codex hook or command-wrapper bundle.
    Codex,
    /// OpenCode plugin or hook-wrapper bundle.
    Opencode,
    /// Claude Code hook-wrapper bundle.
    Claude,
    /// Pi agent hook-wrapper bundle.
    Pi,
}

/// Optional repo-native content template for the first-run onboarding flow.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentContentTemplate {
    /// Do not activate content runtime models.
    None,
    /// Activate broad agent-project facts such as decisions and requirements.
    AgentProject,
    /// Activate agent-project facts plus source-document custody metadata.
    DocumentProject,
}

impl AgentContentTemplate {
    /// Stable lowercase template label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::AgentProject => "agent-project",
            Self::DocumentProject => "document-project",
        }
    }

    /// Whether this template enables content runtime configuration.
    pub fn activates_content(self) -> bool {
        !matches!(self, Self::None)
    }
}

impl AgentOnboardingTarget {
    /// Matching integration target when this onboarding target is concrete.
    pub fn integration_target(self) -> Option<AgentIntegrationTarget> {
        match self {
            Self::Auto | Self::Generic => None,
            Self::Codex => Some(AgentIntegrationTarget::Codex),
            Self::Opencode => Some(AgentIntegrationTarget::Opencode),
            Self::Claude => Some(AgentIntegrationTarget::Claude),
            Self::Pi => Some(AgentIntegrationTarget::Pi),
        }
    }

    /// Stable lowercase target label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Generic => "generic",
            Self::Codex => "codex",
            Self::Opencode => "opencode",
            Self::Claude => "claude",
            Self::Pi => "pi",
        }
    }
}

impl AgentIntegrationTarget {
    /// Stable lowercase target label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Opencode => "opencode",
            Self::Claude => "claude",
            Self::Pi => "pi",
        }
    }

    /// Matching nudge target.
    pub fn nudge_target(self) -> AgentNudgeTarget {
        match self {
            Self::Codex => AgentNudgeTarget::Codex,
            Self::Opencode => AgentNudgeTarget::Opencode,
            Self::Claude => AgentNudgeTarget::Claude,
            Self::Pi => AgentNudgeTarget::Pi,
        }
    }
}

/// Local agent event that can receive a bounded Assura nudge.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentNudgeEvent {
    /// Compact session-start health summary.
    SessionStart,
    /// Before a likely file-inspection or file-editing tool call.
    BeforeTool,
    /// After a tool call changed or inspected relevant files.
    AfterTool,
    /// Before or after a read-focused file event.
    FileRead,
    /// Recovery or resume event after tool failure, stale state, or context loss.
    Recovery,
}

/// Agent host label for documentation and hook routing.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentNudgeTarget {
    /// Vendor-neutral JSON/text nudge.
    Generic,
    /// Codex hook or tool wrapper.
    Codex,
    /// OpenCode plugin or hook wrapper.
    Opencode,
    /// Claude Code hook wrapper.
    Claude,
    /// Pi agent hook wrapper.
    Pi,
}
