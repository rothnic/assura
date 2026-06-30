//! Local editor-facing project-intelligence command dispatch.

use super::{EditorCommands, ExitCode};
use std::path::PathBuf;

/// Run an editor-facing project-intelligence command.
pub async fn editor_command(command: EditorCommands, config: Option<PathBuf>) -> ExitCode {
    match command {
        EditorCommands::Session { path } => {
            crate::cli::content_query::editor_session_command(path, config)
        }
    }
}
