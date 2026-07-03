//! Content query context loading and error handling.

use super::ContentCommands;
use crate::cli::check::{run_structure_check, StructureCheckReport};
use crate::cli::AgentQueryArg;
use crate::cli::ExitCode;
use crate::config::config::Config;
use crate::config::loader::ConfigLoader;
use crate::content_repository::{ContentFinding, ContentRepository, RepositoryModel};
use crate::intelligence::{FactIngestor, InMemoryFactStore};
use crate::markdown_links::is_markdown_file;
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_REFERENCE_FILE_LIMIT: usize = 512;
const SOURCE_REFERENCE_FILE_SIZE_LIMIT: u64 = 512 * 1024;

#[derive(Debug)]
pub(crate) struct QueryContext {
    pub(crate) project_root: PathBuf,
    pub(crate) config_path: PathBuf,
    pub(crate) store: InMemoryFactStore,
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
            command.references_enabled(),
        )
    }

    pub(crate) fn load_for_path(
        path: PathBuf,
        config: Option<PathBuf>,
        semantic_enabled: bool,
        code_symbols_enabled: bool,
        references_enabled: bool,
    ) -> Result<Self, ContentQueryError> {
        let (config_path, project_root) = match config {
            Some(config_path) => {
                let config_path = config_path.canonicalize().map_err(|error| {
                    ContentQueryError::runtime(format!(
                        "failed to resolve config path {}: {error}",
                        config_path.display()
                    ))
                })?;
                let checked_path = path.canonicalize().map_err(|error| {
                    ContentQueryError::runtime(format!(
                        "failed to resolve project path {}: {error}",
                        path.display()
                    ))
                })?;
                let project_root = project_root_for_explicit_config(&checked_path, &config_path)?;
                (config_path, project_root)
            }
            None => {
                let config_path =
                    crate::cli::ConfigDiscovery::find_config_path(&path).ok_or_else(|| {
                        ContentQueryError::no_config(format!(
                            "no .assura/config.yml found for {}",
                            path.display()
                        ))
                    })?;
                let project_root = config_path
                    .parent()
                    .and_then(std::path::Path::parent)
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or_else(|| path.clone());
                (config_path, project_root)
            }
        };
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
        ingest_markdown_reference_facts(&mut ingestor, &project_root);
        if references_enabled {
            ingest_source_reference_facts(&mut ingestor, &project_root);
            ingest_frontmatter_reference_facts(&mut ingestor, &project_root, &config);
        }
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

fn project_root_for_explicit_config(
    checked_path: &Path,
    config_path: &Path,
) -> Result<PathBuf, ContentQueryError> {
    let assura_dir = config_path.parent().ok_or_else(|| {
        ContentQueryError::configuration(format!(
            "invalid Assura config location {}",
            config_path.display()
        ))
    })?;
    let project_root = if assura_dir.file_name().and_then(|name| name.to_str()) == Some(".assura") {
        assura_dir.parent().ok_or_else(|| {
            ContentQueryError::configuration(format!(
                "invalid Assura config location {}",
                config_path.display()
            ))
        })?
    } else if checked_path.is_file() {
        checked_path.parent().unwrap_or(checked_path)
    } else {
        checked_path
    };
    project_root.canonicalize().map_err(|error| {
        ContentQueryError::runtime(format!(
            "failed to resolve project root {}: {error}",
            project_root.display()
        ))
    })
}

fn ingest_markdown_reference_facts(ingestor: &mut FactIngestor, project_root: &Path) {
    for path in markdown_reference_files(project_root) {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(rel_path) = path.strip_prefix(project_root) else {
            continue;
        };
        ingestor.ingest_markdown_links(project_root, rel_path, &content);
    }
}

fn ingest_source_reference_facts(ingestor: &mut FactIngestor, project_root: &Path) {
    for path in source_reference_files(project_root)
        .into_iter()
        .take(SOURCE_REFERENCE_FILE_LIMIT)
    {
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.len() > SOURCE_REFERENCE_FILE_SIZE_LIMIT {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(rel_path) = path.strip_prefix(project_root) else {
            continue;
        };
        ingestor.ingest_source_references(project_root, rel_path, &content);
    }
}

fn ingest_frontmatter_reference_facts(
    ingestor: &mut FactIngestor,
    project_root: &Path,
    config: &Config,
) {
    let Some(extensions) = &config.extensions else {
        return;
    };
    let field_policies = extensions
        .repository_references
        .iter()
        .filter(|policy| !policy.frontmatter_fields.is_empty())
        .collect::<Vec<_>>();
    if field_policies.is_empty() {
        return;
    }
    for path in markdown_reference_files(project_root) {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(rel_path) = path.strip_prefix(project_root) else {
            continue;
        };
        let fields = field_policies
            .iter()
            .filter(|policy| repository_reference_policy_matches(&policy.paths, rel_path))
            .flat_map(|policy| policy.frontmatter_fields.iter().cloned())
            .collect::<Vec<_>>();
        if fields.is_empty() {
            continue;
        }
        ingestor.ingest_frontmatter_references(project_root, rel_path, &content, &fields);
    }
}

fn markdown_reference_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown_reference_files(project_root, project_root, &mut files);
    files.sort();
    files
}

fn source_reference_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_source_reference_files(project_root, project_root, &mut files);
    files.sort();
    files
}

fn collect_markdown_reference_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        let Ok(rel_path) = path.strip_prefix(root) else {
            continue;
        };
        if ignored_reference_scan_path(rel_path) {
            continue;
        }
        if file_type.is_dir() {
            collect_markdown_reference_files(root, &path, files);
        } else if file_type.is_file() && is_markdown_file(&path) {
            files.push(path);
        }
    }
}

fn collect_source_reference_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        let Ok(rel_path) = path.strip_prefix(root) else {
            continue;
        };
        if ignored_reference_scan_path(rel_path) {
            continue;
        }
        if file_type.is_dir() {
            collect_source_reference_files(root, &path, files);
        } else if file_type.is_file() && is_source_reference_file(&path) {
            files.push(path);
        }
    }
}

pub(super) fn ignored_reference_scan_path(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        matches!(
            value.as_ref(),
            ".git" | "target" | "node_modules" | "dist" | "coverage"
        )
    })
}

fn repository_reference_policy_matches(patterns: &[String], rel: &Path) -> bool {
    if patterns.is_empty() {
        return true;
    }
    let rel = rel.to_string_lossy().replace('\\', "/");
    patterns
        .iter()
        .any(|pattern| glob::Pattern::new(pattern).is_ok_and(|pattern| pattern.matches(&rel)))
}

fn is_source_reference_file(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "rs" | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "h"
            | "hpp"
            | "cpp"
            | "cs"
            | "rb"
            | "php"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "toml"
            | "yaml"
            | "yml"
            | "json"
            | "jsonl"
    )
}

fn ingest_safe_fix_structure_report(ingestor: &mut FactIngestor, mut report: StructureCheckReport) {
    report.violations.retain(|violation| {
        violation.rule == "markdown_trailing_spaces"
            || violation.rule.starts_with("requirements_traceability:")
            || violation.rule.starts_with("computed_check:")
    });
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
        | ContentCommands::References { path, .. }
        | ContentCommands::Expand { path, .. } => path.clone(),
    }
}

trait SemanticCommand {
    fn semantic_enabled(&self) -> bool;
    fn code_symbols_enabled(&self) -> bool;
    fn references_enabled(&self) -> bool;
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

    fn references_enabled(&self) -> bool {
        matches!(
            self,
            ContentCommands::AgentContext { .. }
                | ContentCommands::AgentQuery {
                    query: AgentQueryArg::GraphExpand
                        | AgentQueryArg::UnresolvedReferences
                        | AgentQueryArg::Gaps
                        | AgentQueryArg::NextActions,
                    ..
                }
                | ContentCommands::ContextPack { .. }
                | ContentCommands::Session { .. }
                | ContentCommands::References { .. }
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
pub(crate) struct ContentQueryError {
    pub(crate) message: String,
    pub(crate) exit_code: ExitCode,
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
