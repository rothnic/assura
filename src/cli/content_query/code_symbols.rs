//! Code-symbol query helpers for fact-backed content commands.

use super::context::{ContentQueryError, QueryContext};
use super::facts::{fact_by_id, fact_kind, fact_path};
use super::output::{SymbolRefOutput, SymbolRefsOutput, SymbolsOutput};
use crate::intelligence::{model_instance_id, FactId, ProjectFact, SymbolRef};
use std::path::PathBuf;

pub(super) fn symbols_for_instance(
    context: &QueryContext,
    collection: &str,
    id: &str,
) -> Result<SymbolsOutput, ContentQueryError> {
    let source_id = model_instance_id(collection, id);
    if fact_by_id(context, &source_id).is_none() {
        return Err(ContentQueryError::runtime(format!(
            "content instance not found: {collection}/{id}"
        )));
    }
    let symbols = context
        .store
        .symbol_refs()
        .into_iter()
        .filter(|edge| edge.source_id == source_id)
        .map(|edge| symbol_ref_output(context, edge))
        .collect();

    Ok(SymbolsOutput {
        collection: collection.to_string(),
        id: id.to_string(),
        source_id: source_id.to_string(),
        symbols,
    })
}

pub(super) fn symbol_refs(context: &QueryContext, symbol: &str) -> SymbolRefsOutput {
    let references = context
        .store
        .symbol_refs()
        .into_iter()
        .filter(|edge| {
            symbol_text_matches(&edge.symbol, symbol)
                || edge
                    .target_id
                    .as_ref()
                    .and_then(|target_id| fact_by_id(context, target_id))
                    .is_some_and(|fact| match fact {
                        ProjectFact::CodeSymbol(code_symbol) => {
                            symbol_text_matches(&code_symbol.symbol, symbol)
                        }
                        _ => false,
                    })
        })
        .map(|edge| symbol_ref_output(context, edge))
        .collect();

    SymbolRefsOutput {
        symbol: symbol.to_string(),
        references,
    }
}

fn symbol_ref_output(context: &QueryContext, edge: &SymbolRef) -> SymbolRefOutput {
    let (source_kind, collection, instance_id, source_path) =
        symbol_source_output(context, &edge.source_id);
    let target_fact = edge
        .target_id
        .as_ref()
        .and_then(|target_id| fact_by_id(context, target_id));
    let (target_symbol, target_path, evidence) = match target_fact {
        Some(ProjectFact::CodeSymbol(symbol)) => (
            Some(symbol.symbol.clone()),
            symbol
                .location
                .as_ref()
                .map(|location| location.path.clone()),
            Some(symbol.evidence.clone()),
        ),
        _ => (None, None, None),
    };

    SymbolRefOutput {
        source_id: edge.source_id.to_string(),
        source_kind,
        collection,
        instance_id,
        source_path,
        field: edge.field.clone(),
        symbol: edge.symbol.clone(),
        provider: edge.provider.clone(),
        resolved: edge.target_id.is_some(),
        target_id: edge.target_id.as_ref().map(ToString::to_string),
        target_symbol,
        target_path,
        evidence,
    }
}

fn symbol_source_output(
    context: &QueryContext,
    source_id: &FactId,
) -> (String, Option<String>, Option<String>, Option<PathBuf>) {
    match fact_by_id(context, source_id) {
        Some(ProjectFact::ModelInstance(instance)) => (
            "model_instance".to_string(),
            Some(instance.collection.clone()),
            Some(instance.instance_id.clone()),
            fact_path(context, &ProjectFact::ModelInstance(instance.clone())),
        ),
        Some(fact) => (
            fact_kind(fact).to_string(),
            None,
            None,
            fact_path(context, fact),
        ),
        None => ("unknown".to_string(), None, None, None),
    }
}

fn symbol_text_matches(candidate: &str, requested: &str) -> bool {
    candidate == requested
        || candidate.rsplit("::").next() == Some(requested)
        || requested.rsplit("::").next() == Some(candidate)
}
