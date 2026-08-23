use super::ingest::FactIngestor;
use super::set::model_instance_id;
use super::types::{EdgeId, FactGeneration, FactId, FactOrigin, ProjectFact, SourceLocation};
use crate::content_repository::{CodeSymbolSpec, RepositoryModel, RepositoryValidation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Built-in no-dependency Rust symbol extraction baseline.
pub const LOCAL_RUST_SYMBOL_PROVIDER: &str = "rust-token-baseline-v1";

/// Code symbol fact, optionally imported from a future provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeSymbol {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Provider-specific symbol identifier.
    pub symbol: String,
    /// Provider that produced this symbol, such as a local baseline or SCIP import.
    pub provider: String,
    /// Provider evidence quality, for example `baseline` or `provider`.
    pub evidence: String,
    /// Source location when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

/// Provider-level evidence for code symbol facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeProviderEvidence {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Provider or import name.
    pub provider: String,
    /// Evidence quality, such as `baseline`, `imported`, or `provider`.
    pub quality: String,
    /// Human-readable provider summary.
    pub summary: String,
}

/// Optional edge from a source fact to a code symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRef {
    /// Stable edge ID.
    pub id: EdgeId,
    /// Generation that produced this edge.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Source fact that mentioned the symbol.
    pub source_id: FactId,
    /// Symbol text or provider ID.
    pub symbol: String,
    /// Source field that carried the symbol reference when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Resolved code symbol fact ID when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<FactId>,
    /// Provider name when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl FactIngestor {
    /// Ingest rough local Rust declaration symbols without requiring a provider.
    pub fn ingest_local_rust_code_symbols(&mut self, project_root: &Path) {
        self.facts
            .upsert_fact(ProjectFact::CodeProviderEvidence(CodeProviderEvidence {
                id: FactId::from_parts(
                    "code_provider",
                    &format!("{LOCAL_RUST_SYMBOL_PROVIDER}:baseline"),
                ),
                generation: self.generation.clone(),
                origin: FactOrigin::Derived,
                provider: LOCAL_RUST_SYMBOL_PROVIDER.to_string(),
                quality: "baseline".to_string(),
                summary: "Local token scan of Rust declarations; candidate code context only"
                    .to_string(),
            }));

        for path in rust_source_files(project_root) {
            let rel_path = path
                .strip_prefix(project_root)
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| path.clone());
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            for (symbol, line) in rust_declaration_symbols(&content) {
                self.facts.upsert_fact(ProjectFact::CodeSymbol(CodeSymbol {
                    id: code_symbol_id(
                        LOCAL_RUST_SYMBOL_PROVIDER,
                        &symbol,
                        &format!("{}:{line}", rel_path.display()),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    symbol,
                    provider: LOCAL_RUST_SYMBOL_PROVIDER.to_string(),
                    evidence: "baseline".to_string(),
                    location: Some(
                        SourceLocation::path(rel_path.clone()).with_position(Some(line), None),
                    ),
                }));
            }
        }
    }

    /// Ingest configured content fields as unresolved or resolved symbol refs.
    pub fn ingest_content_code_symbol_refs(
        &mut self,
        model: &RepositoryModel,
        validation: &RepositoryValidation,
    ) {
        for collection in &model.collections {
            if collection.code_symbols.is_empty() {
                continue;
            }
            for object in validation
                .snapshot
                .objects
                .values()
                .filter(|object| object.collection == collection.name)
            {
                let source_id = model_instance_id(&object.collection, &object.id);
                for spec in &collection.code_symbols {
                    for symbol in code_symbol_values(&object.data, spec) {
                        let target_id = spec
                            .provider
                            .as_ref()
                            .and_then(|provider| self.resolve_code_symbol(provider, &symbol));
                        self.add_symbol_ref_with_field(
                            source_id.clone(),
                            symbol,
                            Some(spec.field.clone()),
                            spec.provider.clone(),
                            target_id,
                        );
                    }
                }
            }
        }
    }

    fn resolve_code_symbol(&self, provider: &str, requested: &str) -> Option<FactId> {
        let matches = self
            .facts
            .facts
            .iter()
            .filter_map(|fact| match fact {
                ProjectFact::CodeSymbol(symbol)
                    if symbol.provider == provider && symbol_matches(&symbol.symbol, requested) =>
                {
                    Some(symbol.id.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            matches.into_iter().next()
        } else {
            None
        }
    }
}

fn code_symbol_id(provider: &str, symbol: &str, evidence_key: &str) -> FactId {
    FactId::from_parts(
        "code_symbol",
        &format!("{provider}:{symbol}:{evidence_key}"),
    )
}

fn code_symbol_values(data: &serde_json::Map<String, Value>, spec: &CodeSymbolSpec) -> Vec<String> {
    let Some(value) = data.get(&spec.field) else {
        return Vec::new();
    };
    match value {
        Value::String(value) if !value.trim().is_empty() => vec![value.trim().to_string()],
        Value::Array(values) => values
            .iter()
            .filter_map(|value| match value {
                Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn symbol_matches(candidate: &str, requested: &str) -> bool {
    candidate == requested
        || requested.rsplit("::").next() == Some(candidate)
        || candidate.rsplit("::").next() == Some(requested)
}

fn rust_source_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(project_root, project_root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(rel_path) = path.strip_prefix(root) else {
            continue;
        };
        if ignored_code_symbol_path(rel_path) {
            continue;
        }
        if path.is_dir() {
            collect_rust_source_files(root, &path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn ignored_code_symbol_path(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        matches!(value.as_ref(), ".git" | "target" | "node_modules")
    })
}

fn rust_declaration_symbols(content: &str) -> Vec<(String, usize)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| rust_declaration_symbol(line).map(|symbol| (symbol, index + 1)))
        .collect()
}

fn rust_declaration_symbol(line: &str) -> Option<String> {
    let line = line.trim_start();
    let line = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub(super) "))
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
    for keyword in ["struct", "enum", "trait", "fn", "type"] {
        if let Some(rest) = line
            .strip_prefix(keyword)
            .and_then(|rest| rest.strip_prefix(' '))
        {
            return symbol_identifier(rest);
        }
    }
    None
}

fn symbol_identifier(value: &str) -> Option<String> {
    let identifier = value
        .trim_start()
        .trim_start_matches('<')
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|part| !part.is_empty())?;
    Some(identifier.to_string())
}
