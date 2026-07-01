//! Agent-facing project-intelligence command arguments.

use super::args::OutputFormat;
use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
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

    #[command(about = "Run a persistent JSON-line project-intelligence query session")]
    Session { path: Option<PathBuf> },
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
