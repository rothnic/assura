//! Shared agent-query envelope over fact-backed content query outputs.

use super::code_symbols::{symbol_refs, symbols_for_instance};
use super::context::{ContentQueryError, QueryContext};
use super::output::{
    AgentQueryOutput, AgentQueryRequestOutput, DiagnosticsOutput, SafeFixOutput, SafeFixesOutput,
};
use super::references::{repository_references, ReferenceMode};
use super::{diagnostic_output, expand, missing_relations, search};
use crate::cli::AgentQueryArg as QueryArg;
use crate::intelligence::{ProjectFact, SafeFix};
use crate::stable_hash::stable_hash;
use serde::Serialize;
use std::path::Path;

const AGENT_QUERY_SCHEMA: &str = "assura.project-intelligence.agent-query.v1";

pub(super) fn agent_query(
    context: &QueryContext,
    request: AgentQueryRequest<'_>,
) -> Result<AgentQueryOutput, ContentQueryError> {
    let response = match request.query {
        QueryArg::Capabilities => to_response(agent_query_capabilities())?,
        QueryArg::Diagnostics => to_response(diagnostics(context))?,
        QueryArg::SafeFixes => to_response(safe_fixes(context))?,
        QueryArg::GraphExpand => {
            let collection = required_arg(request.collection, "collection", request.query)?;
            let id = required_arg(request.id, "id", request.query)?;
            to_response(expand(context, collection, id, request.limit)?)?
        }
        QueryArg::KeywordSearch => {
            let text = required_arg(request.text, "text", request.query)?;
            to_response(search(context, text))?
        }
        QueryArg::SemanticCandidates => {
            let text = required_arg(request.text, "text", request.query)?;
            to_response(super::semantic::semantic_search(
                context,
                text,
                request.limit,
                request.enable_local,
            ))?
        }
        QueryArg::CodeSymbols => {
            let collection = required_arg(request.collection, "collection", request.query)?;
            let id = required_arg(request.id, "id", request.query)?;
            to_response(symbols_for_instance(context, collection, id)?)?
        }
        QueryArg::CodeSymbolRefs => {
            let symbol = required_arg(request.symbol, "symbol", request.query)?;
            to_response(symbol_refs(context, symbol))?
        }
        QueryArg::MissingRelations => to_response(missing_relations(context))?,
        QueryArg::UnresolvedReferences => to_response(repository_references(
            context,
            ReferenceMode {
                source: None,
                target: None,
                all: false,
                unresolved: true,
            },
            request.limit,
        )?)?,
        QueryArg::Gaps => to_response(gaps(context))?,
        QueryArg::NextActions => to_response(next_actions(context))?,
    };

    Ok(AgentQueryOutput {
        schema: AGENT_QUERY_SCHEMA,
        request: AgentQueryRequestOutput {
            capability: agent_query_capability(request.query),
            cli: agent_query_cli(request.query),
            project_root: context.project_root.clone(),
            config_path: context.config_path.clone(),
        },
        response,
    })
}

pub(super) struct AgentQueryRequest<'a> {
    pub(super) query: QueryArg,
    pub(super) collection: Option<&'a String>,
    pub(super) id: Option<&'a String>,
    pub(super) text: Option<&'a String>,
    pub(super) symbol: Option<&'a String>,
    pub(super) limit: usize,
    pub(super) enable_local: bool,
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
        QueryArg::Capabilities => "capabilities",
        QueryArg::Diagnostics => "diagnostics",
        QueryArg::SafeFixes => "safe_fixes",
        QueryArg::GraphExpand => "graph_queries",
        QueryArg::KeywordSearch => "keyword_search",
        QueryArg::SemanticCandidates => "semantic_candidates",
        QueryArg::CodeSymbols | QueryArg::CodeSymbolRefs => "code_symbols",
        QueryArg::MissingRelations => "graph_queries",
        QueryArg::UnresolvedReferences => "repository_references",
        QueryArg::Gaps => "gaps",
        QueryArg::NextActions => "next_actions",
    }
}

fn agent_query_cli(query: QueryArg) -> &'static str {
    match query {
        QueryArg::Capabilities => "assura content agent-query capabilities",
        QueryArg::Diagnostics => "assura content agent-query diagnostics",
        QueryArg::SafeFixes => "assura content agent-query safe-fixes",
        QueryArg::GraphExpand => "assura content agent-query graph-expand",
        QueryArg::KeywordSearch => "assura content agent-query keyword-search",
        QueryArg::SemanticCandidates => "assura content agent-query semantic-candidates",
        QueryArg::CodeSymbols => "assura content agent-query code-symbols",
        QueryArg::CodeSymbolRefs => "assura content agent-query code-symbol-refs",
        QueryArg::MissingRelations => "assura content agent-query missing-relations",
        QueryArg::UnresolvedReferences => "assura content agent-query unresolved-references",
        QueryArg::Gaps => "assura content agent-query gaps",
        QueryArg::NextActions => "assura content agent-query next-actions",
    }
}

#[derive(Debug, Serialize)]
struct AgentQueryCapabilitiesOutput {
    capabilities: Vec<AgentQueryCapabilityOutput>,
}

#[derive(Debug, Serialize)]
struct AgentQueryCapabilityOutput {
    name: &'static str,
    description: &'static str,
    required_args: Vec<&'static str>,
    suggested_commands: Vec<&'static str>,
}

fn agent_query_capabilities() -> AgentQueryCapabilitiesOutput {
    AgentQueryCapabilitiesOutput {
        capabilities: vec![
            capability(
                "capabilities",
                "List deterministic project-intelligence capabilities.",
                vec![],
                vec!["assura content agent-query capabilities --format json"],
            ),
            capability(
                "diagnostics",
                "Return fact-backed validation diagnostics.",
                vec![],
                vec!["assura content agent-query diagnostics --format json"],
            ),
            capability(
                "safe_fixes",
                "Return deterministic safe-fix proposals.",
                vec![],
                vec!["assura content agent-query safe-fixes --format json"],
            ),
            capability(
                "keyword_search",
                "Search modeled project-intelligence facts by keyword.",
                vec!["--text"],
                vec!["assura content agent-query keyword-search --text <query> --format json"],
            ),
            capability(
                "semantic_candidates",
                "Return optional local semantic candidate context; candidates do not decide validation truth.",
                vec!["--text"],
                vec![
                    "assura content agent-query semantic-candidates --text <query> --enable-local --format json",
                ],
            ),
            capability(
                "repository_references",
                "Enumerate repository-reference edges and unresolved targets.",
                vec![],
                vec![
                    "assura content references --all --format json",
                    "assura content agent-query unresolved-references --format json",
                ],
            ),
            capability(
                "graph_expand",
                "Expand bounded graph context around one modeled content instance.",
                vec!["--collection", "--id"],
                vec!["assura content agent-query graph-expand --collection <name> --id <id> --format json"],
            ),
            capability(
                "missing_relations",
                "List modeled content relation edges with unresolved targets.",
                vec![],
                vec![
                    "assura content agent-query missing-relations --format json",
                ],
            ),
            capability(
                "code_symbols",
                "Return code-symbol references for one modeled content instance.",
                vec!["--collection", "--id"],
                vec![
                    "assura content agent-query code-symbols --collection <name> --id <id> --format json",
                ],
            ),
            capability(
                "code_symbol_refs",
                "Return modeled content references to one code symbol.",
                vec!["--symbol"],
                vec!["assura content agent-query code-symbol-refs --symbol <symbol> --format json"],
            ),
            capability(
                "gaps",
                "Summarize deterministic gaps an agent should inspect next.",
                vec![],
                vec!["assura content agent-query gaps --format json"],
            ),
            capability(
                "next_actions",
                "Return deterministic follow-up commands for current gaps.",
                vec![],
                vec!["assura content agent-query next-actions --format json"],
            ),
        ],
    }
}

fn capability(
    name: &'static str,
    description: &'static str,
    required_args: Vec<&'static str>,
    suggested_commands: Vec<&'static str>,
) -> AgentQueryCapabilityOutput {
    AgentQueryCapabilityOutput {
        name,
        description,
        required_args,
        suggested_commands,
    }
}

#[derive(Debug, Serialize)]
struct AgentQueryGapsOutput {
    diagnostics: usize,
    safe_fixes: usize,
    missing_relations: usize,
    unresolved_repository_references: usize,
}

fn gaps(context: &QueryContext) -> AgentQueryGapsOutput {
    AgentQueryGapsOutput {
        diagnostics: diagnostics(context).diagnostics.len(),
        safe_fixes: safe_fixes(context).safe_fixes.len(),
        missing_relations: context.store.missing_relationship_targets().len(),
        unresolved_repository_references: context.store.unresolved_repository_references().len(),
    }
}

#[derive(Debug, Serialize)]
struct AgentQueryNextActionsOutput {
    actions: Vec<AgentQueryNextActionOutput>,
}

#[derive(Debug, Serialize)]
struct AgentQueryNextActionOutput {
    reason: &'static str,
    command: &'static str,
}

fn next_actions(context: &QueryContext) -> AgentQueryNextActionsOutput {
    let gaps = gaps(context);
    let mut actions = Vec::new();
    if gaps.diagnostics > 0 {
        actions.push(next_action(
            "validation diagnostics exist",
            "assura content agent-query diagnostics --format json",
        ));
    }
    if gaps.safe_fixes > 0 {
        actions.push(next_action(
            "safe fixes are available",
            "assura content agent-query safe-fixes --format json",
        ));
    }
    if gaps.missing_relations > 0 {
        actions.push(next_action(
            "modeled content relations have unresolved targets",
            "assura content agent-query missing-relations --format json",
        ));
    }
    if gaps.unresolved_repository_references > 0 {
        actions.push(next_action(
            "repository references have unresolved targets",
            "assura content agent-query unresolved-references --format json",
        ));
    }
    if actions.is_empty() {
        actions.push(next_action(
            "no deterministic gaps found",
            "assura content agent-query capabilities --format json",
        ));
    }
    AgentQueryNextActionsOutput { actions }
}

fn next_action(reason: &'static str, command: &'static str) -> AgentQueryNextActionOutput {
    AgentQueryNextActionOutput { reason, command }
}

pub(super) fn diagnostics(context: &QueryContext) -> DiagnosticsOutput {
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

pub(super) fn safe_fixes(context: &QueryContext) -> SafeFixesOutput {
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
        audit_id: safe_fix_audit_id(fix),
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

fn safe_fix_audit_id(fix: &SafeFix) -> Option<String> {
    let location = fix.location.as_ref()?;
    let line = location.line?;
    if fix.operation != "remove_blank_line_trailing_spaces" {
        return None;
    }
    let key = format!(
        "{}:{}:{}",
        fix.operation,
        portable_path(&location.path),
        line
    );
    Some(format!(
        "markdown.safe_fix.{:016x}",
        stable_hash(key.as_bytes())
    ))
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
