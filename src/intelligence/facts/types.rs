use super::code_symbols::{CodeProviderEvidence, CodeSymbol, SymbolRef};
use crate::stable_hash::stable_hash;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use std::path::PathBuf;

/// Stable identifier for one project intelligence fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FactId(String);

impl FactId {
    /// Build a stable fact ID from a kind namespace and deterministic key.
    pub fn from_parts(kind: &str, key: &str) -> Self {
        Self(format!("{kind}:{:016x}", stable_hash(key.as_bytes())))
    }

    /// Return the string representation used by serialized facts.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable identifier for one project intelligence edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EdgeId(String);

impl EdgeId {
    /// Build a stable edge ID from a kind namespace and deterministic key.
    pub fn from_parts(kind: &str, key: &str) -> Self {
        Self(format!("{kind}:{:016x}", stable_hash(key.as_bytes())))
    }

    /// Return the string representation used by serialized edges.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Generation or snapshot label used to replace facts from one ingest run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactGeneration {
    /// Caller-provided generation identifier.
    pub id: String,
}

impl FactGeneration {
    /// Create a generation label.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Whether a fact was read from source material or derived by Assura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactOrigin {
    /// Fact comes directly from repository files or configuration.
    Source,
    /// Fact was computed from source facts, validation, or indexing logic.
    Derived,
}

/// Source location that lets diagnostics and fixes attach back to files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Repository-relative path.
    pub path: PathBuf,
    /// One-based line number when known.
    pub line: Option<usize>,
    /// One-based column number when known.
    pub column: Option<usize>,
    /// Model or frontmatter field when known.
    pub field: Option<String>,
}

impl SourceLocation {
    /// Create a source location for a repository-relative path.
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            line: None,
            column: None,
            field: None,
        }
    }

    /// Attach a one-based line and column.
    pub fn with_position(mut self, line: Option<usize>, column: Option<usize>) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    /// Attach a model or frontmatter field.
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// Collection model loaded from Assura content runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDefinition {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Content runtime collection name.
    pub collection: String,
    /// Runtime object type or schema class.
    pub object_type: String,
    /// Backing adapter name.
    pub adapter: String,
}

/// Field definition discovered from runtime schema artifacts or collection IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Owning model fact ID.
    pub model_id: FactId,
    /// Field name.
    pub name: String,
    /// JSON-ish field kind when known.
    pub kind: String,
    /// Whether the field is required by the runtime schema or ID contract.
    pub required: bool,
}

/// Relationship definition loaded from content runtime relation config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipDefinition {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Owning model fact ID.
    pub model_id: FactId,
    /// Source field that carries the reference.
    pub field: String,
    /// Target collection names.
    pub target_collections: Vec<String>,
    /// Whether the reference can contain many targets.
    pub many: bool,
    /// Whether the reference is required.
    pub required: bool,
    /// Whether the relation must be acyclic.
    pub acyclic: bool,
}

/// Path pattern scope that maps repository files into a collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathScope {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Owning model fact ID.
    pub model_id: FactId,
    /// Collection name.
    pub collection: String,
    /// Glob pattern from the content runtime config.
    pub pattern: String,
}

/// Repository resource such as a Markdown, JSON, YAML, or JSONL file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Repository-relative path.
    pub path: PathBuf,
    /// File extension when present.
    pub extension: Option<String>,
}

/// Markdown document fact for a resource with Markdown body content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownDocument {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Backing resource fact ID.
    pub resource_id: FactId,
    /// Repository-relative path.
    pub path: PathBuf,
}

/// Markdown heading section fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownSection {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Owning Markdown document fact ID.
    pub document_id: FactId,
    /// Heading level.
    pub level: usize,
    /// Heading text.
    pub title: String,
    /// One-based line number.
    pub line_number: usize,
}

/// Typed repository object instance loaded from a content collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInstance {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Owning model fact ID.
    pub model_id: FactId,
    /// Backing resource fact ID.
    pub resource_id: FactId,
    /// Collection name.
    pub collection: String,
    /// Object type.
    pub object_type: String,
    /// Instance ID inside the collection.
    pub instance_id: String,
    /// Raw object data used by current content runtime validation.
    pub data: Map<String, Value>,
}

/// Validation diagnostic attached to a resource or model instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Diagnostic rule or code.
    pub rule: String,
    /// Severity label.
    pub severity: String,
    /// Human-readable message.
    pub message: String,
    /// Target fact ID when Assura can resolve one.
    pub target_id: Option<FactId>,
    /// Source location for editor and agent surfaces.
    pub location: Option<SourceLocation>,
}

/// Deterministic safe-fix proposal attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeFix {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Diagnostic this fix addresses.
    pub diagnostic_id: FactId,
    /// Target fact ID when Assura can resolve one.
    pub target_id: Option<FactId>,
    /// Source location for the concrete edit when known.
    pub location: Option<SourceLocation>,
    /// Machine-readable operation name.
    pub operation: String,
    /// Human-readable summary.
    pub summary: String,
}

/// Text-search chunk derived from a model instance or Markdown section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchChunk {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Source fact that produced this chunk.
    pub source_id: FactId,
    /// Text content for keyword or semantic indexing.
    pub text: String,
}

/// Optional embedding metadata for a search chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Source search chunk fact ID.
    pub chunk_id: FactId,
    /// Embedding provider or model identifier.
    pub provider: String,
    /// Stable hash of the source chunk text that produced this vector.
    pub text_hash: String,
    /// Number of vector dimensions expected for this provider output.
    pub dimensions: usize,
    /// Embedding vector when available.
    pub vector: Vec<f32>,
}

/// Derived relation edge between content runtime model instances.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipEdge {
    /// Stable edge ID.
    pub id: EdgeId,
    /// Generation that produced this edge.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Source model instance fact ID.
    pub source_id: FactId,
    /// Target model instance fact ID when resolved.
    pub target_id: Option<FactId>,
    /// Source field carrying the relation.
    pub field: String,
    /// Target collections allowed by the relationship.
    pub target_collections: Vec<String>,
    /// Target object ID from source data.
    pub target_instance_id: String,
}

/// Node-like project intelligence fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum ProjectFact {
    /// Content runtime collection model.
    ModelDefinition(ModelDefinition),
    /// Field on a model.
    FieldDefinition(FieldDefinition),
    /// Relation configuration on a model.
    RelationshipDefinition(RelationshipDefinition),
    /// Collection path scope.
    PathScope(PathScope),
    /// Repository resource.
    Resource(Resource),
    /// Markdown document.
    MarkdownDocument(MarkdownDocument),
    /// Markdown heading section.
    MarkdownSection(MarkdownSection),
    /// Typed content runtime instance.
    ModelInstance(ModelInstance),
    /// Validation diagnostic.
    Diagnostic(Diagnostic),
    /// Safe fix proposal.
    SafeFix(SafeFix),
    /// Searchable text chunk.
    SearchChunk(SearchChunk),
    /// Optional embedding record.
    EmbeddingRecord(EmbeddingRecord),
    /// Optional code symbol.
    CodeSymbol(CodeSymbol),
    /// Code-symbol provider provenance.
    CodeProviderEvidence(CodeProviderEvidence),
}

impl ProjectFact {
    /// Stable fact ID.
    pub fn id(&self) -> &FactId {
        match self {
            Self::ModelDefinition(fact) => &fact.id,
            Self::FieldDefinition(fact) => &fact.id,
            Self::RelationshipDefinition(fact) => &fact.id,
            Self::PathScope(fact) => &fact.id,
            Self::Resource(fact) => &fact.id,
            Self::MarkdownDocument(fact) => &fact.id,
            Self::MarkdownSection(fact) => &fact.id,
            Self::ModelInstance(fact) => &fact.id,
            Self::Diagnostic(fact) => &fact.id,
            Self::SafeFix(fact) => &fact.id,
            Self::SearchChunk(fact) => &fact.id,
            Self::EmbeddingRecord(fact) => &fact.id,
            Self::CodeSymbol(fact) => &fact.id,
            Self::CodeProviderEvidence(fact) => &fact.id,
        }
    }

    /// Generation that produced this fact.
    pub fn generation(&self) -> &FactGeneration {
        match self {
            Self::ModelDefinition(fact) => &fact.generation,
            Self::FieldDefinition(fact) => &fact.generation,
            Self::RelationshipDefinition(fact) => &fact.generation,
            Self::PathScope(fact) => &fact.generation,
            Self::Resource(fact) => &fact.generation,
            Self::MarkdownDocument(fact) => &fact.generation,
            Self::MarkdownSection(fact) => &fact.generation,
            Self::ModelInstance(fact) => &fact.generation,
            Self::Diagnostic(fact) => &fact.generation,
            Self::SafeFix(fact) => &fact.generation,
            Self::SearchChunk(fact) => &fact.generation,
            Self::EmbeddingRecord(fact) => &fact.generation,
            Self::CodeSymbol(fact) => &fact.generation,
            Self::CodeProviderEvidence(fact) => &fact.generation,
        }
    }
}

/// Edge-like project intelligence fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum ProjectEdge {
    /// Content runtime relationship between model instances.
    Relationship(RelationshipEdge),
    /// Optional reference from any fact to a code symbol.
    SymbolRef(SymbolRef),
}

impl ProjectEdge {
    /// Stable edge ID.
    pub fn id(&self) -> &EdgeId {
        match self {
            Self::Relationship(edge) => &edge.id,
            Self::SymbolRef(edge) => &edge.id,
        }
    }

    /// Generation that produced this edge.
    pub fn generation(&self) -> &FactGeneration {
        match self {
            Self::Relationship(edge) => &edge.generation,
            Self::SymbolRef(edge) => &edge.generation,
        }
    }
}
