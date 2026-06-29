//! Shared agent-query envelope over fact-backed content query outputs.

use super::code_symbols::{symbol_refs, symbols_for_instance};
use super::context::{ContentQueryError, QueryContext};
use super::output::{
    AgentQueryOutput, AgentQueryRequestOutput, DiagnosticsOutput, SafeFixOutput, SafeFixesOutput,
};
use super::{diagnostic_output, expand, missing_relations, search};
use crate::cli::AgentQueryArg as QueryArg;
use crate::intelligence::{ProjectFact, SafeFix};
use serde::Serialize;

const AGENT_QUERY_SCHEMA: &str = "assura.project-intelligence.agent-query.v1";

pub(super) fn agent_query(
    context: &QueryContext,
    query: QueryArg,
    collection: Option<&String>,
    id: Option<&String>,
    text: Option<&String>,
    symbol: Option<&String>,
    limit: usize,
    enable_local: bool,
) -> Result<AgentQueryOutput, ContentQueryError> {
    let response = match query {
        QueryArg::Diagnostics => to_response(diagnostics(context))?,
        QueryArg::SafeFixes => to_response(safe_fixes(context))?,
        QueryArg::GraphExpand => {
            let collection = required_arg(collection, "collection", query)?;
            let id = required_arg(id, "id", query)?;
            to_response(expand(context, collection, id, limit)?)?
        }
        QueryArg::KeywordSearch => {
            let text = required_arg(text, "text", query)?;
            to_response(search(context, text))?
        }
        QueryArg::SemanticCandidates => {
            let text = required_arg(text, "text", query)?;
            to_response(super::semantic::semantic_search(
                context,
                text,
                limit,
                enable_local,
            ))?
        }
        QueryArg::CodeSymbols => {
            let collection = required_arg(collection, "collection", query)?;
            let id = required_arg(id, "id", query)?;
            to_response(symbols_for_instance(context, collection, id)?)?
        }
        QueryArg::CodeSymbolRefs => {
            let symbol = required_arg(symbol, "symbol", query)?;
            to_response(symbol_refs(context, symbol))?
        }
        QueryArg::MissingRelations => to_response(missing_relations(context))?,
    };

    Ok(AgentQueryOutput {
        schema: AGENT_QUERY_SCHEMA,
        request: AgentQueryRequestOutput {
            capability: agent_query_capability(query),
            cli: agent_query_cli(query),
            project_root: context.project_root.clone(),
            config_path: context.config_path.clone(),
        },
        response,
    })
}

fn required_arg<'a>(
    value: Option<&'a String>,
    name: &str,
    query: QueryArg,
) -> Result<&'a str, ContentQueryError> {
    value
        .map(String::as_str)
        .ok_or_else(|| ContentQueryError::configuration(format!("{query:?} requires --{name}")))
}

fn to_response<T: Serialize>(value: T) -> Result<serde_json::Value, ContentQueryError> {
    serde_json::to_value(value).map_err(|error| ContentQueryError::runtime(error.to_string()))
}

fn agent_query_capability(query: QueryArg) -> &'static str {
    match query {
        QueryArg::Diagnostics => "diagnostics",
        QueryArg::SafeFixes => "safe_fixes",
        QueryArg::GraphExpand => "graph_queries",
        QueryArg::KeywordSearch => "keyword_search",
        QueryArg::SemanticCandidates => "semantic_candidates",
        QueryArg::CodeSymbols | QueryArg::CodeSymbolRefs => "code_symbols",
        QueryArg::MissingRelations => "graph_queries",
    }
}

fn agent_query_cli(query: QueryArg) -> &'static str {
    match query {
        QueryArg::Diagnostics => "assura content agent-query diagnostics",
        QueryArg::SafeFixes => "assura content agent-query safe-fixes",
        QueryArg::GraphExpand => "assura content agent-query graph-expand",
        QueryArg::KeywordSearch => "assura content agent-query keyword-search",
        QueryArg::SemanticCandidates => "assura content agent-query semantic-candidates",
        QueryArg::CodeSymbols => "assura content agent-query code-symbols",
        QueryArg::CodeSymbolRefs => "assura content agent-query code-symbol-refs",
        QueryArg::MissingRelations => "assura content agent-query missing-relations",
    }
}

fn diagnostics(context: &QueryContext) -> DiagnosticsOutput {
    let diagnostics = context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::Diagnostic(diagnostic) => Some(diagnostic_output(diagnostic)),
            _ => None,
        })
        .collect();
    DiagnosticsOutput { diagnostics }
}

fn safe_fixes(context: &QueryContext) -> SafeFixesOutput {
    let safe_fixes = context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::SafeFix(fix) => Some(safe_fix_output(fix)),
            _ => None,
        })
        .collect();
    SafeFixesOutput { safe_fixes }
}

fn safe_fix_output(fix: &SafeFix) -> SafeFixOutput {
    SafeFixOutput {
        id: fix.id.to_string(),
        diagnostic_id: fix.diagnostic_id.to_string(),
        target_id: fix.target_id.as_ref().map(ToString::to_string),
        operation: fix.operation.clone(),
        summary: fix.summary.clone(),
        path: fix.location.as_ref().map(|location| location.path.clone()),
        line: fix.location.as_ref().and_then(|location| location.line),
        column: fix.location.as_ref().and_then(|location| location.column),
        field: fix
            .location
            .as_ref()
            .and_then(|location| location.field.clone()),
    }
}
