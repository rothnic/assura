//! Local agent-facing project-intelligence command dispatch.

use super::{AgentCommands, ContentCommands, ExitCode};
use std::path::PathBuf;

/// Run local project-intelligence commands for coding agents.
pub async fn agent_command(command: AgentCommands, config: Option<PathBuf>) -> ExitCode {
    crate::cli::content_query::content_command(agent_to_content_command(command), config).await
}

fn agent_to_content_command(command: AgentCommands) -> ContentCommands {
    match command {
        AgentCommands::Context { path, format } => ContentCommands::AgentContext { path, format },
        AgentCommands::Diagnostics { path, format } => ContentCommands::AgentQuery {
            query: crate::cli::AgentQueryArg::Diagnostics,
            path,
            collection: None,
            id: None,
            text: None,
            symbol: None,
            limit: 20,
            enable_local: false,
            format,
        },
        AgentCommands::ContextPack {
            path,
            collection,
            id,
            text,
            limit,
            format,
        } => ContentCommands::ContextPack {
            path,
            collection,
            id,
            text,
            limit,
            format,
        },
        AgentCommands::Show {
            collection,
            id,
            path,
            format,
        } => ContentCommands::Show {
            collection,
            id,
            path,
            format,
        },
        AgentCommands::Search {
            query,
            path,
            format,
        } => ContentCommands::Search {
            query,
            path,
            format,
        },
        AgentCommands::MissingRelations { path, format } => {
            ContentCommands::MissingRelations { path, format }
        }
        AgentCommands::Expand {
            collection,
            id,
            path,
            limit,
            format,
        } => ContentCommands::Expand {
            collection,
            id,
            path,
            limit,
            format,
        },
        AgentCommands::SafeFixes { path, format } => ContentCommands::AgentQuery {
            query: crate::cli::AgentQueryArg::SafeFixes,
            path,
            collection: None,
            id: None,
            text: None,
            symbol: None,
            limit: 20,
            enable_local: false,
            format,
        },
        AgentCommands::Session { path } => ContentCommands::Session { path },
    }
}
