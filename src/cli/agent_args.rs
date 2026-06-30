//! Agent-facing project-intelligence command arguments.

use super::args::OutputFormat;
use clap::Subcommand;
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

    #[command(about = "Run a persistent JSON-line project-intelligence query session")]
    Session { path: Option<PathBuf> },
}
