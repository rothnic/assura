//! Shared project-intelligence contract for agent and editor wrappers.

use super::{FactSet, ProjectEdge, ProjectFact};
use serde::Serialize;

/// Stable schema for a project-intelligence agent context response.
pub const PROJECT_INTELLIGENCE_AGENT_CONTEXT_SCHEMA: &str =
    "assura.project-intelligence.agent-context.v1";

/// Agent-facing project-intelligence context summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectIntelligenceAgentContext {
    /// Stable response schema.
    pub schema: &'static str,
    /// Capability records wrappers can map to CLI, LSP, MCP, or daemon tools.
    pub capabilities: Vec<AgentSurfaceCapability>,
    /// Current fact and edge counts behind those capabilities.
    pub summary: AgentSurfaceSummary,
}

/// One shared capability exposed through the project-intelligence contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentSurfaceCapability {
    /// Capability key.
    pub name: &'static str,
    /// Current support level.
    pub status: &'static str,
    /// Current CLI command that proves the behavior.
    pub cli: &'static str,
    /// Stable contract note for wrappers.
    pub contract: &'static str,
}

/// Count summary for current project-intelligence facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentSurfaceSummary {
    /// Typed content model instances.
    pub model_instances: usize,
    /// Validation diagnostics.
    pub diagnostics: usize,
    /// Deterministic safe fixes.
    pub safe_fixes: usize,
    /// Relationship edges.
    pub relationship_edges: usize,
    /// Relationship edges without a resolved target.
    pub unresolved_relationship_edges: usize,
    /// Repository-internal reference edges.
    pub repository_reference_edges: usize,
    /// Repository-internal reference edges without a resolved target path.
    pub unresolved_repository_reference_edges: usize,
    /// Search chunks available for keyword or semantic retrieval.
    pub search_chunks: usize,
    /// Optional embedding records.
    pub embedding_records: usize,
    /// Code-symbol reference edges.
    pub symbol_refs: usize,
    /// Code-symbol reference edges with resolved symbol facts.
    pub resolved_symbol_refs: usize,
}

/// Build the shared project-intelligence agent context from current facts.
pub fn project_intelligence_agent_context(facts: &FactSet) -> ProjectIntelligenceAgentContext {
    ProjectIntelligenceAgentContext {
        schema: PROJECT_INTELLIGENCE_AGENT_CONTEXT_SCHEMA,
        capabilities: vec![
            capability(
                "diagnostics",
                "supported",
                "assura check --format agent",
                "Validation diagnostics use the shared agent feedback schema.",
            ),
            capability(
                "safe_fixes",
                "supported",
                "assura fix markdown",
                "Safe fixes are deterministic and bounded to explicit operations.",
            ),
            capability(
                "agent_queries",
                "supported",
                "assura content agent-query",
                "Agent-query envelopes wrap shared diagnostics, graph, search, semantic, code-symbol, and safe-fix result contracts.",
            ),
            capability(
                "graph_queries",
                "supported",
                "assura content expand",
                "Graph context comes from local project-intelligence facts.",
            ),
            capability(
                "keyword_search",
                "supported",
                "assura content search",
                "Keyword search is deterministic local text matching.",
            ),
            capability(
                "semantic_candidates",
                "supported_optional",
                "assura content semantic-search --enable-local",
                "Semantic scores are candidate context only, not validation truth.",
            ),
            capability(
                "code_symbols",
                "supported_optional",
                "assura content symbols",
                "Code-symbol refs are optional and remain queryable when unresolved.",
            ),
        ],
        summary: summarize_agent_surface(facts),
    }
}

fn capability(
    name: &'static str,
    status: &'static str,
    cli: &'static str,
    contract: &'static str,
) -> AgentSurfaceCapability {
    AgentSurfaceCapability {
        name,
        status,
        cli,
        contract,
    }
}

fn summarize_agent_surface(facts: &FactSet) -> AgentSurfaceSummary {
    let mut summary = AgentSurfaceSummary {
        model_instances: 0,
        diagnostics: 0,
        safe_fixes: 0,
        relationship_edges: 0,
        unresolved_relationship_edges: 0,
        repository_reference_edges: 0,
        unresolved_repository_reference_edges: 0,
        search_chunks: 0,
        embedding_records: 0,
        symbol_refs: 0,
        resolved_symbol_refs: 0,
    };

    for fact in &facts.facts {
        match fact {
            ProjectFact::ModelInstance(_) => summary.model_instances += 1,
            ProjectFact::Diagnostic(_) => summary.diagnostics += 1,
            ProjectFact::SafeFix(_) => summary.safe_fixes += 1,
            ProjectFact::SearchChunk(_) => summary.search_chunks += 1,
            ProjectFact::EmbeddingRecord(_) => summary.embedding_records += 1,
            _ => {}
        }
    }
    for edge in &facts.edges {
        match edge {
            ProjectEdge::Relationship(edge) => {
                summary.relationship_edges += 1;
                if edge.target_id.is_none() {
                    summary.unresolved_relationship_edges += 1;
                }
            }
            ProjectEdge::RepositoryReference(edge) => {
                summary.repository_reference_edges += 1;
                if edge.target_id.is_none() {
                    summary.unresolved_repository_reference_edges += 1;
                }
            }
            ProjectEdge::SymbolRef(edge) => {
                summary.symbol_refs += 1;
                if edge.target_id.is_some() {
                    summary.resolved_symbol_refs += 1;
                }
            }
        }
    }
    summary
}
