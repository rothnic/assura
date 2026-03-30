pub mod cli;
pub mod config;
pub mod constraints;
pub mod intelligence;
pub mod markdown;
pub mod maturity;
pub mod validation;
pub mod ls_compat;

pub use constraints::{
    Constraint, ConstraintConfig, ConstraintContext, ConstraintEngine, ConstraintError,
    ConstraintOutput, ConstraintResult, ConstraintTrigger, DirectoryConstraint, DirectoryRule,
    DirectoryValidationConfig, CaseConvention, ExtensionRule, ExtensionPattern,
    FileChangeTrigger, FileSizeConstraint, FileSizeLimit, FileSizeRule, ManualTrigger, 
    MaturityTrigger, MultiPartExtensionRule, MultipleRuleSyntax, NamingConstraint, 
    NamingPattern, NamingRule, PathRule, PathRuleConfig, Severity, SeverityConfig, 
    SeverityMapping, TriggerRegistry, ValidationFailure, ValidationFailures,
};

pub use intelligence::{
    GraphBuilder, GraphError, GraphPersistence, GraphQuery, GraphResult, IntelligenceGraph,
    Node, NodeId, NodeMetadata, NodeType, Relationship,
};

pub use markdown::{
    FieldType, FieldValidator, FrontmatterConstraint, FrontmatterSchema,
    HeadingConstraint, parser::Heading, parser::HeadingHierarchy, HeadingLevel, headings::HeadingPattern, HeadingStructure,
    HeadingValidator, MarkdownConstraint, MarkdownDocument, MarkdownError, MarkdownParser,
    MarkdownResult, MarkdownSchema, MarkdownValidationError, MarkdownValidationRule,
    SchemaDefinition, SectionDefinition, SectionValidator, TemplateConstraint, TemplateDefinition,
    headings::TextPatternRule, ValidationConfig,
};

pub use maturity::{
    MaturityConfig, MaturityDecisionEngine, MaturityDetector, MaturityError, MaturityLevel,
    MaturityReport, MaturityResult, MaturitySignal, SignalCollector, SignalPipeline, SignalType,
};
