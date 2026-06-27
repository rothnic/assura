//! Data model for the repo-native content repository prototype.

use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    MarkdownFrontmatter,
    JsonRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    String,
    StringArray,
    Enum(Vec<String>),
}

impl FieldKind {
    pub(super) fn validate(&self, value: &Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::StringArray => value
                .as_array()
                .is_some_and(|items| items.iter().all(Value::is_string)),
            Self::Enum(values) => value
                .as_str()
                .is_some_and(|actual| values.iter().any(|expected| expected == actual)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: String,
    pub kind: FieldKind,
    pub required: bool,
}

impl FieldSpec {
    pub fn required(name: impl Into<String>, kind: FieldKind) -> Self {
        Self {
            name: name.into(),
            kind,
            required: true,
        }
    }

    pub fn optional(name: impl Into<String>, kind: FieldKind) -> Self {
        Self {
            name: name.into(),
            kind,
            required: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSpec {
    pub field: String,
    pub target_collection: String,
    pub many: bool,
}

impl ReferenceSpec {
    pub fn many(field: impl Into<String>, target_collection: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            target_collection: target_collection.into(),
            many: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSpec {
    pub name: String,
    pub object_type: String,
    pub path_pattern: String,
    pub adapter: AdapterKind,
    pub id_field: String,
    pub fields: Vec<FieldSpec>,
    pub references: Vec<ReferenceSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementRule {
    pub directory: PathBuf,
    pub allowed_types: HashSet<String>,
    pub recursive: bool,
}

impl PlacementRule {
    pub fn recursive(
        directory: impl Into<PathBuf>,
        allowed_types: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            directory: directory.into(),
            allowed_types: allowed_types.into_iter().map(Into::into).collect(),
            recursive: true,
        }
    }

    pub(super) fn allows(&self, rel_path: &Path, object_type: &str) -> bool {
        if !self.allowed_types.contains(object_type) {
            return false;
        }
        if self.recursive {
            rel_path.starts_with(&self.directory)
        } else {
            rel_path.parent() == Some(self.directory.as_path())
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryModel {
    pub collections: Vec<CollectionSpec>,
    pub placements: Vec<PlacementRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoObject {
    pub collection: String,
    pub object_type: String,
    pub id: String,
    pub rel_path: PathBuf,
    pub data: Map<String, Value>,
    pub body: Option<String>,
    pub headings: Vec<MarkdownHeading>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownHeading {
    pub level: usize,
    pub text: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoEdge {
    pub source: ObjectKey,
    pub field: String,
    pub target_collection: String,
    pub target_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    pub collection: String,
    pub id: String,
}

impl ObjectKey {
    pub fn new(collection: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            id: id.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepositorySnapshot {
    pub objects: BTreeMap<(String, String), RepoObject>,
    pub edges: Vec<RepoEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentFinding {
    pub code: &'static str,
    pub path: Option<PathBuf>,
    pub message: String,
}

impl ContentFinding {
    pub(super) fn new(
        code: &'static str,
        path: Option<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryValidation {
    pub snapshot: RepositorySnapshot,
    pub findings: Vec<ContentFinding>,
}
