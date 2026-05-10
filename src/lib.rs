pub mod cli;
pub mod config;
pub mod constraints;
pub mod intelligence;
pub mod ls_compat;
pub mod markdown;
pub mod maturity;
pub mod validation;

pub use constraints::{
    CaseConvention, Constraint, ConstraintConfig, ConstraintContext, ConstraintEngine,
    ConstraintError, ConstraintOutput, ConstraintResult, ConstraintTrigger, DirectoryConstraint,
    DirectoryRule, DirectoryValidationConfig, ExtensionPattern, ExtensionRule, FileChangeTrigger,
    FileSizeConstraint, FileSizeLimit, FileSizeRule, ManualTrigger, MaturityTrigger,
    MultiPartExtensionRule, MultipleRuleSyntax, NamingConstraint, NamingPattern, NamingRule,
    PathRule, PathRuleConfig, Severity, SeverityConfig, SeverityMapping, TriggerRegistry,
    ValidationFailure, ValidationFailures,
};

pub use intelligence::{
    GraphBuilder, GraphError, GraphPersistence, GraphQuery, GraphResult, IntelligenceGraph, Node,
    NodeId, NodeMetadata, NodeType, Relationship,
};

pub use markdown::{
    headings::HeadingPattern, headings::TextPatternRule, parser::Heading, parser::HeadingHierarchy,
    FieldType, FieldValidator, FrontmatterConstraint, FrontmatterSchema, HeadingConstraint,
    HeadingLevel, HeadingStructure, HeadingValidator, MarkdownConstraint, MarkdownDocument,
    MarkdownError, MarkdownParser, MarkdownResult, MarkdownSchema, MarkdownValidationError,
    MarkdownValidationRule, SchemaDefinition, SectionDefinition, SectionValidator,
    TemplateConstraint, TemplateDefinition, ValidationConfig,
};

pub use maturity::{
    MaturityConfig, MaturityDecisionEngine, MaturityDetector, MaturityError, MaturityLevel,
    MaturityReport, MaturityResult, MaturitySignal, SignalCollector, SignalPipeline, SignalType,
};
