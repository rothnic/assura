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
pub(crate) mod policy;
#[cfg(test)]
mod policy_naming_contract_tests {
    use std::collections::HashMap;

    use regex_lite::Regex;

    use crate::policy::naming::{
        validate_file_stem_with_path, validate_name_with_path, validate_single_name_with_path,
    };

    #[test]
    fn policy_naming_preserves_supported_naming_predicates() {
        let regexes = HashMap::<String, Regex>::new();

        assert!(validate_name_with_path(
            "snake_case",
            "src/snake_case.rs",
            "snake_case",
            &regexes
        ));
        assert!(validate_name_with_path(
            "PascalCase",
            "src/PascalCase.rs",
            "PascalCase",
            &regexes
        ));
        assert!(!validate_name_with_path(
            "BadName",
            "src/BadName.rs",
            "kebab-case",
            &regexes
        ));
        assert!(validate_single_name_with_path(
            "scope-spec",
            "scope",
            "regex:^${0}-spec$",
            &regexes
        ));
        assert!(validate_file_stem_with_path(
            "archive.tar",
            "archive.tar.gz",
            "kebab-case",
            &regexes
        ));
        assert!(validate_single_name_with_path(
            "README",
            "README.md",
            "exact:README",
            &regexes
        ));
    }
}
#[cfg(feature = "full-cli")]
pub mod constraints;
#[cfg(feature = "full-cli")]
#[doc(hidden)]
pub mod content_repository;
#[cfg(feature = "full-cli")]
/// Experimental daemon-ready state contracts shared by local integrations.
///
/// These APIs are internal pre-1.0 building blocks until daemon support is
/// promoted in the release surface matrix.
pub mod daemon;
#[cfg(feature = "full-cli")]
/// Experimental dependency-intelligence internals.
///
/// This is not a supported dependency graph validation release surface.
pub mod intelligence;
#[cfg(test)]
mod ls_compat;
#[cfg(feature = "full-cli")]
pub mod markdown;
#[cfg(any(feature = "full-cli", feature = "yaml-config"))]
#[path = "markdown/links.rs"]
pub(crate) mod markdown_links;
#[cfg(feature = "full-cli")]
/// Experimental maturity internals.
///
/// This is not a supported maturity detection release surface.
pub mod maturity;
pub(crate) mod repository_references;
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
    RelationshipEdge, RepositoryReferenceEdge, Resource, SafeFix, SearchChunk, SemanticSearchHit,
    SourceLocation, SymbolRef, LOCAL_HASH_EMBEDDING_DIMENSIONS, LOCAL_HASH_EMBEDDING_PROVIDER,
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

#[cfg(feature = "full-cli")]
pub use maturity::{CiExecutionState, MaturityLevel, ProjectObservations};
