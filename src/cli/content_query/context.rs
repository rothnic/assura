//! Content query context loading and error handling.

use super::ContentCommands;
use crate::cli::ExitCode;
use crate::config::loader::ConfigLoader;
use crate::content_repository::{ContentFinding, ContentRepository, RepositoryModel};
use crate::intelligence::{FactIngestor, InMemoryFactStore};
use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct QueryContext {
    pub(super) project_root: PathBuf,
    pub(super) config_path: PathBuf,
    pub(super) store: InMemoryFactStore,
}

impl QueryContext {
    pub(super) fn load(
        command: &ContentCommands,
        config: Option<PathBuf>,
    ) -> Result<Self, ContentQueryError> {
        let path = match command_path(command) {
            Some(path) => path,
            None => std::env::current_dir().map_err(|error| {
                ContentQueryError::runtime(format!("failed to read current directory: {error}"))
            })?,
        };
        let config_path = match config {
            Some(path) => path,
            None => crate::cli::ConfigDiscovery::find_config_path(&path).ok_or_else(|| {
                ContentQueryError::no_config(format!(
                    "no .assura/config.yml found for {}",
                    path.display()
                ))
            })?,
        };
        let project_root = config_path
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| path.clone());
        let config = ConfigLoader::load(&config_path).map_err(|error| {
            ContentQueryError::configuration(format!(
                "failed to load {}: {error}",
                config_path.display()
            ))
        })?;
        let model = RepositoryModel::from_config(&project_root, &config)
            .map_err(format_content_findings)?;
        let repository =
            ContentRepository::try_new(model.clone()).map_err(format_content_findings)?;
        let validation = repository.validate(&project_root);

        let mut ingestor = FactIngestor::new("content-query");
        ingestor.ingest_repository_model(&model);
        ingestor.ingest_repository_validation(&validation);
        let store = InMemoryFactStore::load(ingestor.finish());

        Ok(Self {
            project_root,
            config_path,
            store,
        })
    }
}

fn command_path(command: &ContentCommands) -> Option<PathBuf> {
    match command {
        ContentCommands::Collections { path, .. }
        | ContentCommands::Instances { path, .. }
        | ContentCommands::Show { path, .. }
        | ContentCommands::Search { path, .. }
        | ContentCommands::MissingRelations { path, .. }
        | ContentCommands::Expand { path, .. } => path.clone(),
    }
}

fn format_content_findings(findings: Vec<ContentFinding>) -> ContentQueryError {
    let message = findings
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("; ");
    ContentQueryError::configuration(message)
}

#[derive(Debug)]
pub(super) struct ContentQueryError {
    pub(super) message: String,
    pub(super) exit_code: ExitCode,
}

impl ContentQueryError {
    pub(super) fn configuration(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: ExitCode::ConfigurationError,
        }
    }

    pub(super) fn no_config(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: ExitCode::NoConfigFound,
        }
    }

    pub(super) fn runtime(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: ExitCode::RuntimeError,
        }
    }
}

impl std::fmt::Display for ContentQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
