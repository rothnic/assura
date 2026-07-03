use super::code_symbols::SymbolRef;
use super::ingest_helpers::{
    adapter_name, column_from_message, line_from_message, schema_fields, searchable_object_text,
};
use super::set::{model_definition_id, model_instance_id, normalize_path, resource_id, FactSet};
use super::types::*;
use crate::cli::StructureCheckReport;
use crate::content_repository::{
    CollectionSpec, ContentFinding, RepositoryModel, RepositoryValidation,
};
use crate::intelligence::semantic::{local_hash_embedding_record, LOCAL_HASH_EMBEDDING_PROVIDER};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

/// Builder that projects current Assura runtime outputs into facts.
#[derive(Debug, Clone)]
pub struct FactIngestor {
    pub(super) generation: FactGeneration,
    pub(super) facts: FactSet,
}

impl FactIngestor {
    /// Create an ingestor for one generation.
    pub fn new(generation: impl Into<String>) -> Self {
        Self {
            generation: FactGeneration::new(generation),
            facts: FactSet::default(),
        }
    }

    /// Ingest content runtime model configuration and schema metadata.
    pub fn ingest_repository_model(&mut self, model: &RepositoryModel) {
        for collection in &model.collections {
            self.ingest_collection_model(collection, model.schema_artifact.as_ref());
        }
    }

    /// Ingest content runtime validation snapshots and findings.
    pub fn ingest_repository_validation(&mut self, validation: &RepositoryValidation) {
        for object in validation.snapshot.objects.values() {
            let resource_id = resource_id(&object.rel_path);
            self.facts.upsert_fact(ProjectFact::Resource(Resource {
                id: resource_id.clone(),
                generation: self.generation.clone(),
                origin: FactOrigin::Source,
                path: object.rel_path.clone(),
                extension: object
                    .rel_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(ToOwned::to_owned),
            }));

            let model_id = model_definition_id(&object.collection, &object.object_type);
            let instance_id = model_instance_id(&object.collection, &object.id);
            self.facts
                .upsert_fact(ProjectFact::ModelInstance(ModelInstance {
                    id: instance_id.clone(),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Source,
                    model_id,
                    resource_id: resource_id.clone(),
                    collection: object.collection.clone(),
                    object_type: object.object_type.clone(),
                    instance_id: object.id.clone(),
                    data: object.data.clone(),
                }));

            self.facts
                .upsert_fact(ProjectFact::SearchChunk(SearchChunk {
                    id: FactId::from_parts(
                        "search_chunk",
                        &format!("instance:{}:{}", object.collection, object.id),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    source_id: instance_id.clone(),
                    text: searchable_object_text(&object.data),
                }));

            if object.body.is_some() {
                let document_id = markdown_document_id(&object.rel_path);
                self.facts
                    .upsert_fact(ProjectFact::MarkdownDocument(MarkdownDocument {
                        id: document_id.clone(),
                        generation: self.generation.clone(),
                        origin: FactOrigin::Source,
                        resource_id,
                        path: object.rel_path.clone(),
                    }));

                for heading in &object.headings {
                    let section_id =
                        markdown_section_id(&object.rel_path, heading.line_number, &heading.text);
                    self.facts
                        .upsert_fact(ProjectFact::MarkdownSection(MarkdownSection {
                            id: section_id.clone(),
                            generation: self.generation.clone(),
                            origin: FactOrigin::Source,
                            document_id: document_id.clone(),
                            level: heading.level,
                            title: heading.text.clone(),
                            line_number: heading.line_number,
                        }));
                    self.facts
                        .upsert_fact(ProjectFact::SearchChunk(SearchChunk {
                            id: FactId::from_parts(
                                "search_chunk",
                                &format!(
                                    "section:{}:{}:{}",
                                    normalize_path(&object.rel_path),
                                    heading.line_number,
                                    heading.text
                                ),
                            ),
                            generation: self.generation.clone(),
                            origin: FactOrigin::Derived,
                            source_id: section_id,
                            text: heading.text.clone(),
                        }));
                }
            }
        }

        for edge in &validation.snapshot.edges {
            let source_id = model_instance_id(&edge.source.collection, &edge.source.id);
            let target_collections = relationship_candidate_collections(validation, edge);
            let target_id = resolved_edge_target(validation, edge, &target_collections);
            self.facts
                .upsert_edge(ProjectEdge::Relationship(RelationshipEdge {
                    id: EdgeId::from_parts(
                        "relationship",
                        &format!(
                            "{}:{}:{}:{}",
                            edge.source.collection, edge.source.id, edge.field, edge.target_id
                        ),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    source_id,
                    target_id,
                    field: edge.field.clone(),
                    target_collections,
                    target_instance_id: edge.target_id.clone(),
                }));
        }

        for finding in &validation.findings {
            let target_id = finding_target_id(validation, finding)
                .or_else(|| finding.path.as_ref().map(resource_id));
            let mut location = finding.path.as_ref().map(SourceLocation::path);
            if let (Some(loc), Some(field)) = (&mut location, finding.field.as_ref()) {
                loc.field = Some(field.clone());
            }
            let diagnostic_id = FactId::from_parts(
                "diagnostic",
                &format!(
                    "content:{}:{}:{}",
                    finding.code,
                    finding
                        .path
                        .as_deref()
                        .map(normalize_path)
                        .unwrap_or_else(|| "-".to_string()),
                    finding.message
                ),
            );
            self.facts.upsert_fact(ProjectFact::Diagnostic(Diagnostic {
                id: diagnostic_id.clone(),
                generation: self.generation.clone(),
                origin: FactOrigin::Derived,
                rule: format!("content_runtime:{}", finding.code),
                severity: "high".to_string(),
                message: finding.message.clone(),
                target_id,
                location,
                metadata: None,
            }));
            self.upsert_diagnostic_search_chunk(diagnostic_id, finding.message.clone());
        }
    }

    /// Ingest structure check diagnostics and known safe-fix proposals.
    pub fn ingest_check_report(&mut self, report: &StructureCheckReport) {
        for violation in &report.violations {
            let target_id = resource_id(&violation.path);
            let location = SourceLocation::path(violation.path.clone()).with_position(
                line_from_message(&violation.message),
                column_from_message(&violation.message),
            );
            let diagnostic_id = FactId::from_parts(
                "diagnostic",
                &format!(
                    "check:{}:{}:{}",
                    violation.rule,
                    normalize_path(&violation.path),
                    violation.message
                ),
            );
            let fix_location = location.clone();
            self.facts.upsert_fact(ProjectFact::Diagnostic(Diagnostic {
                id: diagnostic_id.clone(),
                generation: self.generation.clone(),
                origin: FactOrigin::Derived,
                rule: violation.rule.clone(),
                severity: violation.severity.clone(),
                message: violation.message.clone(),
                target_id: Some(target_id.clone()),
                location: Some(location),
                metadata: violation.metadata.clone(),
            }));
            self.upsert_diagnostic_search_chunk(diagnostic_id.clone(), violation.message.clone());

            if violation.rule == "markdown_trailing_spaces" {
                self.facts.upsert_fact(ProjectFact::SafeFix(SafeFix {
                    id: FactId::from_parts(
                        "safe_fix",
                        &format!("{}:remove_blank_line_trailing_spaces", diagnostic_id),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    diagnostic_id,
                    target_id: Some(target_id),
                    location: Some(fix_location),
                    operation: "remove_blank_line_trailing_spaces".to_string(),
                    summary: "Remove spaces and tabs from otherwise blank Markdown lines"
                        .to_string(),
                }));
            }
        }
    }

    /// Add an unresolved or resolved reference to a code symbol.
    pub fn add_symbol_ref(
        &mut self,
        source_id: FactId,
        symbol: impl Into<String>,
        provider: Option<String>,
        target_id: Option<FactId>,
    ) {
        self.add_symbol_ref_with_field(source_id, symbol, None, provider, target_id);
    }

    /// Add an unresolved or resolved field-backed reference to a code symbol.
    pub fn add_symbol_ref_with_field(
        &mut self,
        source_id: FactId,
        symbol: impl Into<String>,
        field: Option<String>,
        provider: Option<String>,
        target_id: Option<FactId>,
    ) {
        let symbol = symbol.into();
        self.facts.upsert_edge(ProjectEdge::SymbolRef(SymbolRef {
            id: EdgeId::from_parts(
                "symbol_ref",
                &format!(
                    "{}:{}:{}:{}",
                    source_id,
                    field.as_deref().unwrap_or("-"),
                    provider.as_deref().unwrap_or("-"),
                    symbol
                ),
            ),
            generation: self.generation.clone(),
            origin: FactOrigin::Derived,
            source_id,
            symbol,
            field,
            target_id,
            provider,
        }));
    }

    /// Add a precomputed search chunk to this generation.
    pub fn add_search_chunk(&mut self, chunk: SearchChunk) {
        self.facts.upsert_fact(ProjectFact::SearchChunk(chunk));
    }

    /// Finish ingestion and return the fact set.
    pub fn finish(self) -> FactSet {
        self.facts
    }

    /// Add local deterministic embedding records for all current search chunks.
    pub fn ingest_local_semantic_embeddings(&mut self) {
        let chunks = self
            .facts
            .facts
            .iter()
            .filter_map(|fact| match fact {
                ProjectFact::SearchChunk(chunk) => Some(chunk.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        for chunk in chunks {
            let record = local_hash_embedding_record(&chunk);
            debug_assert_eq!(record.provider, LOCAL_HASH_EMBEDDING_PROVIDER);
            self.facts.upsert_fact(ProjectFact::EmbeddingRecord(record));
        }
    }

    fn ingest_collection_model(&mut self, collection: &CollectionSpec, schema: Option<&Value>) {
        let model_id = model_definition_id(&collection.name, &collection.object_type);
        self.facts
            .upsert_fact(ProjectFact::ModelDefinition(ModelDefinition {
                id: model_id.clone(),
                generation: self.generation.clone(),
                origin: FactOrigin::Source,
                collection: collection.name.clone(),
                object_type: collection.object_type.clone(),
                adapter: adapter_name(collection.adapter).to_string(),
            }));
        self.facts.upsert_fact(ProjectFact::PathScope(PathScope {
            id: FactId::from_parts("path_scope", &collection.name),
            generation: self.generation.clone(),
            origin: FactOrigin::Source,
            model_id: model_id.clone(),
            collection: collection.name.clone(),
            pattern: collection.path_pattern.clone(),
        }));

        for (field, kind, required) in schema_fields(collection, schema) {
            self.facts
                .upsert_fact(ProjectFact::FieldDefinition(FieldDefinition {
                    id: FactId::from_parts(
                        "field",
                        &format!("{}:{}:{field}", collection.name, collection.object_type),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Source,
                    model_id: model_id.clone(),
                    name: field,
                    kind,
                    required,
                }));
        }

        for reference in &collection.references {
            self.facts.upsert_fact(ProjectFact::RelationshipDefinition(
                RelationshipDefinition {
                    id: FactId::from_parts(
                        "relationship_definition",
                        &format!(
                            "{}:{}:{}",
                            collection.name, collection.object_type, reference.field
                        ),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Source,
                    model_id: model_id.clone(),
                    field: reference.field.clone(),
                    target_collections: reference.target_collections.clone(),
                    many: reference.many,
                    required: reference.required,
                    acyclic: reference.acyclic,
                },
            ));
        }
    }

    fn upsert_diagnostic_search_chunk(&mut self, diagnostic_id: FactId, text: String) {
        self.facts
            .upsert_fact(ProjectFact::SearchChunk(SearchChunk {
                id: FactId::from_parts("search_chunk", &format!("diagnostic:{diagnostic_id}")),
                generation: self.generation.clone(),
                origin: FactOrigin::Derived,
                source_id: diagnostic_id,
                text,
            }));
    }
}

fn finding_target_id(
    validation: &RepositoryValidation,
    finding: &ContentFinding,
) -> Option<FactId> {
    let path = finding.path.as_ref()?;
    let matches = validation
        .snapshot
        .objects
        .values()
        .filter(|object| {
            object.rel_path == *path
                && finding
                    .object_type
                    .as_ref()
                    .map(|object_type| object_type == &object.object_type)
                    .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [object] => Some(model_instance_id(&object.collection, &object.id)),
        _ => None,
    }
}

fn markdown_document_id(path: impl AsRef<Path>) -> FactId {
    FactId::from_parts("markdown_document", &normalize_path(path.as_ref()))
}

fn markdown_section_id(path: impl AsRef<Path>, line_number: usize, title: &str) -> FactId {
    FactId::from_parts(
        "markdown_section",
        &format!("{}:{line_number}:{title}", normalize_path(path.as_ref())),
    )
}

fn resolved_edge_target(
    validation: &RepositoryValidation,
    edge: &crate::content_repository::RepoEdge,
    candidate_collections: &[String],
) -> Option<FactId> {
    let matches = relationship_target_matches(validation, edge, candidate_collections);
    match matches.as_slice() {
        [collection] => Some(model_instance_id(collection, &edge.target_id)),
        _ => None,
    }
}

fn relationship_target_matches(
    validation: &RepositoryValidation,
    edge: &crate::content_repository::RepoEdge,
    candidate_collections: &[String],
) -> Vec<String> {
    candidate_collections
        .iter()
        .filter(|collection| {
            validation
                .snapshot
                .objects
                .contains_key(&((**collection).clone(), edge.target_id.clone()))
        })
        .cloned()
        .collect()
}

fn relationship_candidate_collections(
    validation: &RepositoryValidation,
    edge: &crate::content_repository::RepoEdge,
) -> Vec<String> {
    if edge.target_collections.is_empty() {
        let mut collections = validation
            .snapshot
            .objects
            .keys()
            .map(|(collection, _)| collection.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        collections.retain(|collection| collection != &edge.source.collection);
        collections
    } else {
        edge.target_collections.clone()
    }
}
