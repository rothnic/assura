//! Assura library entrypoint.
//!
//! The supported pre-1.0 release contract is the CLI, configuration format, and
//! report output documented in `docs/support-policy.md` and
//! `docs/compatibility-and-surface.md`.
//!
//! Full-CLI Rust modules are unstable internal APIs for the current binary,
//! tests, and benchmark harnesses unless a support-policy row says otherwise.
#[doc(hidden)]
pub mod stable_hash {
    pub use assura_stable_hash::*;
}

pub mod cli;
pub mod config;
#[cfg(feature = "full-cli")]
pub mod constraints;
#[cfg(feature = "full-cli")]
#[doc(hidden)]
pub mod content_repository;
#[cfg(feature = "full-cli")]
/// Experimental dependency-intelligence internals.
///
/// This is not a supported dependency graph validation release surface.
pub mod intelligence;
#[cfg(test)]
mod ls_compat;
#[cfg(feature = "full-cli")]
pub mod markdown;
#[cfg(feature = "full-cli")]
/// Experimental maturity internals.
///
/// This is not a supported maturity detection release surface.
pub mod maturity;
#[cfg(feature = "full-cli")]
/// Internal validation engine APIs used by CLI and compatibility tests. These
/// exports do not carry a pre-1.0 compatibility guarantee.
pub mod validation;

#[cfg(feature = "full-cli")]
pub use constraints::{
    CaseConvention, Constraint, ConstraintConfig, ConstraintContext, ConstraintEngine,
    ConstraintError, ConstraintOutput, ConstraintResult, ConstraintTrigger, DirectoryConstraint,
    DirectoryRule, DirectoryValidationConfig, ExtensionPattern, ExtensionRule, FileChangeTrigger,
    FileSizeConstraint, FileSizeLimit, FileSizeRule, ManualTrigger, MaturityTrigger,
    MultiPartExtensionRule, MultipleRuleSyntax, NamingConstraint, NamingPattern, NamingRule,
    PathRule, PathRuleConfig, Severity, SeverityConfig, SeverityMapping, TriggerRegistry,
    ValidationFailure, ValidationFailures,
};

#[cfg(feature = "full-cli")]
pub use intelligence::{
    cosine_similarity, local_hash_embedding, local_hash_embedding_record, model_instance_id,
    project_intelligence_agent_context, semantic_text_hash, AgentSurfaceCapability,
    AgentSurfaceSummary, CodeProviderEvidence, CodeSymbol, Diagnostic, EdgeId, EmbeddingRecord,
    FactGeneration, FactId, FactIngestor, FactOrigin, FactSet, FactStoreStats, FieldDefinition,
    GraphBuilder, GraphError, GraphPersistence, GraphQuery, GraphResult, InMemoryFactStore,
    IntelligenceGraph, MarkdownDocument as ProjectMarkdownDocument, MarkdownLink, MarkdownSection,
    ModelDefinition, ModelInstance, Node, NodeId, NodeMetadata, NodeType, PathScope, ProjectEdge,
    ProjectFact, ProjectIntelligenceAgentContext, Relationship, RelationshipDefinition,
    RelationshipEdge, Resource, SafeFix, SearchChunk, SemanticSearchHit, SourceLocation, SymbolRef,
    LOCAL_HASH_EMBEDDING_DIMENSIONS, LOCAL_HASH_EMBEDDING_PROVIDER,
};

#[cfg(feature = "full-cli")]
pub use markdown::{
    headings::HeadingPattern, headings::TextPatternRule, parser::Heading, parser::HeadingHierarchy,
    FieldType, FieldValidator, FrontmatterConstraint, FrontmatterSchema, HeadingConstraint,
    HeadingLevel, HeadingStructure, HeadingValidator, MarkdownConstraint, MarkdownDocument,
    MarkdownError, MarkdownParser, MarkdownResult, MarkdownSchema, MarkdownValidationError,
    MarkdownValidationRule, SchemaDefinition, SectionDefinition, SectionValidator,
    TemplateConstraint, TemplateDefinition, ValidationConfig,
};

#[cfg(all(feature = "full-cli", feature = "git-signals"))]
pub use maturity::GitSignals;
#[cfg(feature = "full-cli")]
pub use maturity::{
    MaturityConfig, MaturityDecisionEngine, MaturityDetector, MaturityError, MaturityLevel,
    MaturityReport, MaturityResult, MaturitySignal, SignalCollector, SignalPipeline, SignalType,
};
