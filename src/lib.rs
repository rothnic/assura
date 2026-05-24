//! Assura library entrypoint and public module exports.
pub mod cli;
pub mod config;
#[cfg(feature = "full-cli")]
pub mod constraints;
#[cfg(feature = "full-cli")]
pub mod intelligence;
#[cfg(feature = "full-cli")]
pub mod ls_compat;
#[cfg(feature = "full-cli")]
pub mod markdown;
#[cfg(feature = "full-cli")]
pub mod maturity;
#[cfg(feature = "full-cli")]
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
    GraphBuilder, GraphError, GraphPersistence, GraphQuery, GraphResult, IntelligenceGraph, Node,
    NodeId, NodeMetadata, NodeType, Relationship,
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
