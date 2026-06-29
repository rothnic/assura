//! Content query context loading and error handling.

use super::ContentCommands;
use crate::cli::check::{run_structure_check, StructureCheckReport};
use crate::cli::AgentQueryArg;
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
        Self::load_for_path(
            path,
            config,
            command.semantic_enabled(),
            command.code_symbols_enabled(),
        )
    }

    pub(super) fn load_for_path(
        path: PathBuf,
        config: Option<PathBuf>,
        semantic_enabled: bool,
        code_symbols_enabled: bool,
    ) -> Result<Self, ContentQueryError> {
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
        if code_symbols_enabled {
            ingestor.ingest_local_rust_code_symbols(&project_root);
        }
        ingestor.ingest_repository_validation(&validation);
        let structure_report =
            run_structure_check(Some(project_root.clone()), Some(config_path.clone()), false)
                .map_err(|error| {
                    ContentQueryError::runtime(format!(
                        "failed to collect safe-fix structure diagnostics: {error}"
                    ))
                })?;
        ingest_safe_fix_structure_report(&mut ingestor, structure_report);
        if code_symbols_enabled {
            ingestor.ingest_content_code_symbol_refs(&model, &validation);
        }
        if semantic_enabled {
            ingestor.ingest_local_semantic_embeddings();
        }
        let store = InMemoryFactStore::load(ingestor.finish());

        Ok(Self {
            project_root,
            config_path,
            store,
        })
    }
}

fn ingest_safe_fix_structure_report(ingestor: &mut FactIngestor, mut report: StructureCheckReport) {
    report
        .violations
        .retain(|violation| violation.rule == "markdown_trailing_spaces");
    if !report.violations.is_empty() {
        ingestor.ingest_check_report(&report);
    }
}

fn command_path(command: &ContentCommands) -> Option<PathBuf> {
    match command {
        ContentCommands::AgentContext { path, .. }
        | ContentCommands::AgentQuery { path, .. }
        | ContentCommands::ContextPack { path, .. }
        | ContentCommands::Session { path, .. }
        | ContentCommands::Collections { path, .. }
        | ContentCommands::Instances { path, .. }
        | ContentCommands::Show { path, .. }
        | ContentCommands::Search { path, .. }
        | ContentCommands::SemanticSearch { path, .. }
        | ContentCommands::Symbols { path, .. }
        | ContentCommands::SymbolRefs { path, .. }
        | ContentCommands::MissingRelations { path, .. }
        | ContentCommands::Expand { path, .. } => path.clone(),
    }
}

trait SemanticCommand {
    fn semantic_enabled(&self) -> bool;
    fn code_symbols_enabled(&self) -> bool;
}

impl SemanticCommand for ContentCommands {
    fn semantic_enabled(&self) -> bool {
        matches!(
            self,
            ContentCommands::SemanticSearch {
                enable_local: true,
                ..
            } | ContentCommands::AgentQuery {
                query: AgentQueryArg::SemanticCandidates,
                enable_local: true,
                ..
            }
        )
    }

    fn code_symbols_enabled(&self) -> bool {
        matches!(
            self,
            ContentCommands::AgentContext { .. }
                | ContentCommands::AgentQuery {
                    query: AgentQueryArg::GraphExpand
                        | AgentQueryArg::CodeSymbols
                        | AgentQueryArg::CodeSymbolRefs,
                    ..
                }
                | ContentCommands::ContextPack { .. }
                | ContentCommands::Session { .. }
                | ContentCommands::Symbols { .. }
                | ContentCommands::SymbolRefs { .. }
                | ContentCommands::Expand { .. }
        )
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
