//! Agent-query CLI value enums.

use clap::ValueEnum;

/// Project-intelligence capability selector for the shared agent-query envelope.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum AgentQueryArg {
    /// List deterministic project-intelligence capabilities and their arguments.
    Capabilities,
    /// Return fact-backed validation diagnostics.
    Diagnostics,
    /// Return fact-backed deterministic safe-fix proposals.
    SafeFixes,
    /// Expand bounded graph context around one modeled content instance.
    GraphExpand,
    /// Run deterministic keyword search over local project-intelligence facts.
    KeywordSearch,
    /// Run optional local semantic candidate retrieval.
    SemanticCandidates,
    /// Return code symbols referenced by one modeled content instance.
    CodeSymbols,
    /// Return modeled content references to a code symbol.
    CodeSymbolRefs,
    /// Return relationship edges with unresolved targets.
    MissingRelations,
    /// Return repository-reference edges with unresolved targets.
    UnresolvedReferences,
    /// Return deterministic gap summaries for agents.
    Gaps,
    /// Return deterministic next-action suggestions for agents.
    NextActions,
}
