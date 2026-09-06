//! Local agent-facing project-intelligence command dispatch.

use super::agent_integration::agent_integration_command;
use super::agent_nudge::{agent_nudge_command, AgentNudgeOptions};
use super::agent_onboarding::{agent_onboarding_command, AgentOnboardingOptions};
use super::{AgentCommands, ContentCommands, ExitCode};
use std::path::PathBuf;

/// Run local project-intelligence commands for coding agents.
pub async fn agent_command(command: AgentCommands, config: Option<PathBuf>) -> ExitCode {
    match command {
        AgentCommands::Onboard {
            path,
            recipe_file,
            agent,
            activate,
            content_template,
            format,
        } => {
            agent_onboarding_command(
                AgentOnboardingOptions {
                    path,
                    recipe_file,
                    agent,
                    activate,
                    content_template,
                    format,
                },
                config,
            )
            .await
        }
        AgentCommands::Nudge {
            path,
            event,
            changed_paths,
            agent,
            min_severity,
            max_issues,
            reference_limit,
            cooldown_seconds,
            format,
        } => {
            agent_nudge_command(
                AgentNudgeOptions {
                    path,
                    event,
                    changed_paths,
                    agent,
                    min_severity,
                    max_issues,
                    reference_limit,
                    cooldown_seconds,
                    format,
                },
                config,
            )
            .await
        }
        AgentCommands::Integration { command } => agent_integration_command(command).await,
        other => {
            crate::cli::content_query::content_command(agent_to_content_command(other), config)
                .await
        }
    }
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
            raw: false,
            fallback_raw: false,
            limit: 20,
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
        AgentCommands::Onboard { .. } => {
            unreachable!("agent onboard is handled before content routing")
        }
        AgentCommands::Nudge { .. } => {
            unreachable!("agent nudge is handled before content routing")
        }
        AgentCommands::Integration { .. } => {
            unreachable!("agent integration is handled before content routing")
        }
    }
}
