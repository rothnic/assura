//! Fact-backed content query command implementation.

mod agent_query;
mod code_symbols;
mod context;
mod context_pack;
mod editor;
mod editor_protocol;
mod facts;
mod keyword;
mod output;
mod output_text;
mod references;
mod semantic;
mod session;

use self::agent_query::{agent_query, AgentQueryRequest};
use self::code_symbols::{symbol_refs, symbols_for_instance};
use self::context::{ContentQueryError, QueryContext};
use self::context_pack::{context_pack, ContextPackRequest};
pub(crate) use self::editor::editor_session_command;
use self::facts::{
    fact_by_id, fact_kind, fact_path, instance_summary, model_definitions,
    path_scope_for_collection, resources_by_id, sections_for_path,
};
use self::keyword::search;
use self::output::{
    render, CollectionOutput, CollectionsOutput, DiagnosticOutput, ExpandOutput, InstanceOutput,
    InstancesOutput, MissingRelationsOutput, RelatedFactOutput, RelationOutput,
};
use self::references::repository_references;
use self::semantic::semantic_search;
use self::session::content_session_command;
use super::{ContentCommands, ExitCode, OutputFormat};
use crate::intelligence::{
    model_instance_id, project_intelligence_agent_context, Diagnostic, FactId, ProjectEdge,
    ProjectFact, RelationshipEdge,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Run a fact-backed content query command.
pub async fn content_command(command: ContentCommands, config: Option<PathBuf>) -> ExitCode {
    if let ContentCommands::Session { path } = command {
        return content_session_command(path, config);
    }

    match run_content_command(command, config) {
        Ok(rendered) => {
            println!("{rendered}");
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {error}");
            error.exit_code
        }
    }
}

fn run_content_command(
    command: ContentCommands,
    config: Option<PathBuf>,
) -> Result<String, ContentQueryError> {
    let format = command_format(&command);
    let context = QueryContext::load(&command, config)?;
    match command {
        ContentCommands::AgentContext { .. } => render(
            project_intelligence_agent_context(context.store.facts()),
            format,
        ),
        ContentCommands::Session { .. } => unreachable!("session is handled before rendering"),
        ContentCommands::AgentQuery {
            query,
            collection,
            id,
            text,
            symbol,
            limit,
            enable_local,
            ..
        } => render(
            agent_query(
                &context,
                AgentQueryRequest {
                    query,
                    collection: collection.as_ref(),
                    id: id.as_ref(),
                    text: text.as_ref(),
                    symbol: symbol.as_ref(),
                    limit,
                    enable_local,
                },
            )?,
            format,
        ),
        ContentCommands::ContextPack {
            collection,
            id,
            text,
            limit,
            ..
        } => render(
            context_pack(
                &context,
                ContextPackRequest {
                    collection: collection.as_ref(),
                    id: id.as_ref(),
                    text: text.as_ref(),
                    limit,
                },
            )?,
            format,
        ),
        ContentCommands::Collections { .. } => render(collections(&context), format),
        ContentCommands::Instances { collection, .. } => {
            render(instances(&context, &collection), format)
        }
        ContentCommands::Show { collection, id, .. } => {
            render(show_instance(&context, &collection, &id)?, format)
        }
        ContentCommands::Search { query, .. } => render(search(&context, &query), format),
        ContentCommands::SemanticSearch {
            query,
            limit,
            enable_local,
            ..
        } => render(
            semantic_search(&context, &query, limit, enable_local),
            format,
        ),
        ContentCommands::Symbols { collection, id, .. } => {
            render(symbols_for_instance(&context, &collection, &id)?, format)
        }
        ContentCommands::SymbolRefs { symbol, .. } => {
            render(symbol_refs(&context, &symbol), format)
        }
        ContentCommands::MissingRelations { .. } => render(missing_relations(&context), format),
        ContentCommands::References {
            source,
            target,
            limit,
            ..
        } => render(
            repository_references(&context, source.as_ref(), target.as_ref(), limit)?,
            format,
        ),
        ContentCommands::Expand {
            collection,
            id,
            limit,
            ..
        } => render(expand(&context, &collection, &id, limit)?, format),
    }
}

fn command_format(command: &ContentCommands) -> OutputFormat {
    match command {
        ContentCommands::AgentContext { format, .. }
        | ContentCommands::AgentQuery { format, .. }
        | ContentCommands::ContextPack { format, .. }
        | ContentCommands::Collections { format, .. }
        | ContentCommands::Instances { format, .. }
        | ContentCommands::Show { format, .. }
        | ContentCommands::Search { format, .. }
        | ContentCommands::SemanticSearch { format, .. }
        | ContentCommands::Symbols { format, .. }
        | ContentCommands::SymbolRefs { format, .. }
        | ContentCommands::MissingRelations { format, .. }
        | ContentCommands::References { format, .. }
        | ContentCommands::Expand { format, .. } => *format,
        ContentCommands::Session { .. } => OutputFormat::Json,
    }
}

fn collections(context: &QueryContext) -> CollectionsOutput {
    let instance_counts = context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::ModelInstance(instance) => Some(instance.collection.as_str()),
            _ => None,
        })
        .fold(
            BTreeMap::<String, usize>::new(),
            |mut counts, collection| {
                *counts.entry(collection.to_string()).or_default() += 1;
                counts
            },
        );

    let collections = model_definitions(context)
        .into_iter()
        .map(|model| CollectionOutput {
            collection: model.collection.clone(),
            object_type: model.object_type.clone(),
            adapter: model.adapter.clone(),
            path_pattern: path_scope_for_collection(context, &model.collection),
            instances: instance_counts
                .get(&model.collection)
                .copied()
                .unwrap_or_default(),
        })
        .collect();

    CollectionsOutput {
        project_root: context.project_root.clone(),
        config_path: context.config_path.clone(),
        collections,
    }
}

fn instances(context: &QueryContext, collection: &str) -> InstancesOutput {
    let resources = resources_by_id(context.store.facts());
    let mut instances = context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::ModelInstance(instance) if instance.collection == collection => {
                Some(instance_summary(instance, &resources))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    instances.sort_by(|left, right| left.id.cmp(&right.id));

    InstancesOutput {
        collection: collection.to_string(),
        instances,
    }
}

fn show_instance(
    context: &QueryContext,
    collection: &str,
    id: &str,
) -> Result<InstanceOutput, ContentQueryError> {
    let instance_id = model_instance_id(collection, id);
    let instance = context
        .store
        .facts_by_id(&instance_id)
        .into_iter()
        .find_map(|fact| match fact {
            ProjectFact::ModelInstance(instance) => Some(instance),
            _ => None,
        })
        .ok_or_else(|| {
            ContentQueryError::runtime(format!("content instance not found: {collection}/{id}"))
        })?;
    let resources = resources_by_id(context.store.facts());
    let path = resources
        .get(&instance.resource_id)
        .map(|resource| resource.path.clone())
        .unwrap_or_default();

    Ok(InstanceOutput {
        id: instance.instance_id.clone(),
        collection: instance.collection.clone(),
        object_type: instance.object_type.clone(),
        path: path.clone(),
        data: instance.data.clone(),
        outgoing_relations: outgoing_relations(context, &instance_id),
        incoming_relations: incoming_relations(context, &instance_id),
        diagnostics: diagnostics_for_targets(context, &[instance_id, instance.resource_id.clone()]),
        sections: sections_for_path(context, &path),
    })
}

fn outgoing_relations(context: &QueryContext, source_id: &FactId) -> Vec<RelationOutput> {
    let missing = missing_edge_ids(context);
    context
        .store
        .edges_from(source_id)
        .into_iter()
        .filter_map(|edge| match edge {
            ProjectEdge::Relationship(edge) => Some(relation_output(edge, &missing)),
            _ => None,
        })
        .collect()
}

fn incoming_relations(context: &QueryContext, target_id: &FactId) -> Vec<RelationOutput> {
    let missing = missing_edge_ids(context);
    context
        .store
        .facts()
        .edges
        .iter()
        .filter_map(|edge| match edge {
            ProjectEdge::Relationship(edge) if edge.target_id.as_ref() == Some(target_id) => {
                Some(relation_output(edge, &missing))
            }
            _ => None,
        })
        .collect()
}

fn relation_output(edge: &RelationshipEdge, missing: &BTreeSet<String>) -> RelationOutput {
    RelationOutput {
        field: edge.field.clone(),
        source_id: edge.source_id.to_string(),
        target_id: edge.target_id.as_ref().map(ToString::to_string),
        target_instance_id: edge.target_instance_id.clone(),
        target_collections: edge.target_collections.clone(),
        missing: missing.contains(&edge.id.to_string()),
    }
}

fn missing_edge_ids(context: &QueryContext) -> BTreeSet<String> {
    context
        .store
        .missing_relationship_targets()
        .into_iter()
        .map(|edge| edge.id.to_string())
        .collect()
}

fn missing_relations(context: &QueryContext) -> MissingRelationsOutput {
    let missing = missing_edge_ids(context);
    let missing_relations = context
        .store
        .missing_relationship_targets()
        .into_iter()
        .map(|edge| relation_output(edge, &missing))
        .collect();
    MissingRelationsOutput { missing_relations }
}

fn expand(
    context: &QueryContext,
    collection: &str,
    id: &str,
    limit: usize,
) -> Result<ExpandOutput, ContentQueryError> {
    let root_id = model_instance_id(collection, id);
    if fact_by_id(context, &root_id).is_none() {
        return Err(ContentQueryError::runtime(format!(
            "content instance not found: {collection}/{id}"
        )));
    }

    let mut related = Vec::new();
    for edge in context.store.edges_from(&root_id) {
        match edge {
            ProjectEdge::Relationship(edge) => {
                if let Some(target_id) = &edge.target_id {
                    push_related(context, &mut related, target_id, "outgoing_relation", limit);
                }
            }
            ProjectEdge::RepositoryReference(edge) => {
                if let Some(target_id) = &edge.target_id {
                    push_related(
                        context,
                        &mut related,
                        target_id,
                        "repository_reference",
                        limit,
                    );
                }
            }
            ProjectEdge::SymbolRef(edge) => {
                related.push(RelatedFactOutput {
                    id: edge.id.to_string(),
                    kind: "symbol_ref".to_string(),
                    relationship: "symbol_ref".to_string(),
                    path: None,
                });
            }
        }
        if related.len() >= limit {
            break;
        }
    }

    for edge in incoming_relations(context, &root_id) {
        if related.len() >= limit {
            break;
        }
        related.push(RelatedFactOutput {
            id: edge.source_id,
            kind: "relationship_source".to_string(),
            relationship: "incoming_relation".to_string(),
            path: None,
        });
    }

    for diagnostic in diagnostics_for_targets(context, std::slice::from_ref(&root_id)) {
        if related.len() >= limit {
            break;
        }
        related.push(RelatedFactOutput {
            id: diagnostic.id,
            kind: "diagnostic".to_string(),
            relationship: "diagnostic".to_string(),
            path: diagnostic.path,
        });
    }

    related.truncate(limit);
    Ok(ExpandOutput {
        root_id: root_id.to_string(),
        related,
    })
}

fn push_related(
    context: &QueryContext,
    related: &mut Vec<RelatedFactOutput>,
    id: &FactId,
    relationship: &str,
    limit: usize,
) {
    if related.len() >= limit {
        return;
    }
    if let Some(fact) = fact_by_id(context, id) {
        related.push(RelatedFactOutput {
            id: id.to_string(),
            kind: fact_kind(fact).to_string(),
            relationship: relationship.to_string(),
            path: fact_path(context, fact),
        });
    }
}

fn diagnostics_for_targets(context: &QueryContext, targets: &[FactId]) -> Vec<DiagnosticOutput> {
    let targets = targets.iter().collect::<BTreeSet<_>>();
    context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::Diagnostic(diagnostic)
                if diagnostic
                    .target_id
                    .as_ref()
                    .is_some_and(|target| targets.contains(target)) =>
            {
                Some(diagnostic_output(diagnostic))
            }
            _ => None,
        })
        .collect()
}

fn diagnostic_output(diagnostic: &Diagnostic) -> DiagnosticOutput {
    DiagnosticOutput {
        id: diagnostic.id.to_string(),
        rule: diagnostic.rule.clone(),
        severity: diagnostic.severity.clone(),
        message: diagnostic.message.clone(),
        path: diagnostic
            .location
            .as_ref()
            .map(|location| location.path.clone()),
    }
}
