//! Editor-facing project-intelligence command arguments.

use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum EditorCommands {
    #[command(about = "Run a local JSON-line project-intelligence editor session")]
    Session {
        #[arg(help = "Project root or file path (defaults to current directory)")]
        path: Option<PathBuf>,
    },
}
