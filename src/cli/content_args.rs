//! Command-line arguments for content and project-intelligence queries.

use super::agent_query_args::AgentQueryArg;
use super::args::OutputFormat;
use clap::Subcommand;
use std::path::PathBuf;

/// Fact-backed content and project-intelligence commands.
#[derive(Subcommand, Debug)]
pub enum ContentCommands {
    #[command(about = "Report shared project-intelligence agent context")]
    AgentContext {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Run one project-intelligence query through the shared agent envelope")]
    AgentQuery {
        #[arg(value_enum, help = "Shared project-intelligence capability to query")]
        query: AgentQueryArg,
        path: Option<PathBuf>,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        enable_local: bool,
        #[arg(short, long, value_enum, default_value = "text")]
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
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Run a persistent JSON-line project-intelligence query session")]
    Session { path: Option<PathBuf> },

    #[command(about = "List modeled content collections")]
    Collections {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "List instances in one modeled collection")]
    Instances {
        collection: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Show one modeled content instance")]
    Show {
        collection: String,
        id: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Search modeled content facts by keyword")]
    Search {
        query: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Search modeled content facts by optional local semantic candidates")]
    SemanticSearch {
        query: String,
        path: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        enable_local: bool,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    #[command(about = "Report code symbols referenced by a modeled content instance")]
    Symbols {
        collection: String,
        id: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    #[command(about = "Report modeled content instances related to a code symbol")]
    SymbolRefs {
        symbol: String,
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    #[command(about = "Report relationship edges with missing targets")]
    MissingRelations {
        path: Option<PathBuf>,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    #[command(about = "Report repository references by changed source or target path")]
    References {
        path: Option<PathBuf>,
        #[arg(long)]
        source: Option<PathBuf>,
        #[arg(long)]
        target: Option<PathBuf>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    #[command(about = "Expand bounded graph context around one content instance")]
    Expand {
        collection: String,
        id: String,
        path: Option<PathBuf>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}
