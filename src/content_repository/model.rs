//! Data model for repo-native content runtime validation.

use crate::config::config::Config;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    MarkdownFrontmatter,
    JsonRecord,
    YamlRecord,
    JsonlRecord,
}

impl AdapterKind {
    pub(super) fn from_config(value: &str) -> Result<Self, String> {
        match value {
            "markdown_frontmatter" => Ok(Self::MarkdownFrontmatter),
            "json_record" => Ok(Self::JsonRecord),
            "yaml_record" => Ok(Self::YamlRecord),
            "jsonl_record" => Ok(Self::JsonlRecord),
            _ => Err(format!(
                "Unsupported content adapter '{value}'. Supported adapters are markdown_frontmatter, json_record, yaml_record, and jsonl_record"
            )),
        }
    }

    pub(super) fn is_multi_record(self) -> bool {
        matches!(self, Self::JsonlRecord)
    }
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
    pub target_collections: Vec<String>,
    pub many: bool,
    pub required: bool,
    pub acyclic: bool,
}

impl ReferenceSpec {
    pub fn many(field: impl Into<String>, target_collection: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            target_collections: vec![target_collection.into()],
            many: true,
            required: false,
            acyclic: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSpec {
    pub name: String,
    pub object_type: String,
    pub schema_class: Option<String>,
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
    pub schema_artifact_path: Option<PathBuf>,
    pub schema_artifact: Option<Value>,
}

impl RepositoryModel {
    /// Build a repo-native content model from parsed Assura configuration.
    pub fn from_config(project_root: &Path, config: &Config) -> Result<Self, Vec<ContentFinding>> {
        let mut findings = Vec::new();
        if !config.collections.is_empty() && config.models.is_none() {
            findings.push(ContentFinding::new(
                "content_schema_missing",
                None,
                "Configured content collections require models.validation_artifact",
            ));
        }
        let schema_artifact_path = config.models.as_ref().and_then(|models| {
            project_relative_path(
                &models.validation_artifact,
                "models.validation_artifact",
                "content_schema_path_escape",
                &mut findings,
            )
        });
        let schema_artifact = schema_artifact_path
            .as_ref()
            .and_then(|path| load_schema_artifact(project_root, path, &mut findings));

        let mut collections = Vec::new();
        let references = references_by_collection(config, &mut findings);
        let mut collection_names = config.collections.keys().cloned().collect::<Vec<_>>();
        collection_names.sort();
        for name in collection_names {
            let collection = &config.collections[&name];
            let adapter = match AdapterKind::from_config(&collection.adapter) {
                Ok(adapter) => adapter,
                Err(message) => {
                    findings.push(ContentFinding::new(
                        "invalid_content_adapter",
                        None,
                        message,
                    ));
                    continue;
                }
            };
            collections.push(CollectionSpec {
                name: name.clone(),
                object_type: collection.class_name.clone(),
                schema_class: Some(collection.class_name.clone()),
                path_pattern: collection.path.clone(),
                adapter,
                id_field: collection.id.clone(),
                fields: Vec::new(),
                references: references.get(&name).cloned().unwrap_or_default(),
            });
        }

        if schema_artifact_path.is_some() && schema_artifact.is_none() {
            return Err(findings);
        }

        if findings.is_empty() {
            Ok(Self {
                placements: collections
                    .iter()
                    .map(|collection| {
                        PlacementRule::recursive(
                            literal_pattern_prefix(&collection.path_pattern),
                            [collection.object_type.clone()],
                        )
                    })
                    .collect(),
                collections,
                schema_artifact_path,
                schema_artifact,
            })
        } else {
            Err(findings)
        }
    }
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
    pub target_collections: Vec<String>,
    pub target_id: String,
    pub acyclic: bool,
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
    pub object_type: Option<String>,
    pub field: Option<String>,
    pub referenced_object: Option<String>,
    pub message: String,
}

impl ContentFinding {
    pub fn new(code: &'static str, path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            code,
            path,
            object_type: None,
            field: None,
            referenced_object: None,
            message: message.into(),
        }
    }

    pub fn with_object_type(mut self, object_type: impl Into<String>) -> Self {
        self.object_type = Some(object_type.into());
        self
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_referenced_object(mut self, referenced_object: impl Into<String>) -> Self {
        self.referenced_object = Some(referenced_object.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryValidation {
    pub snapshot: RepositorySnapshot,
    pub findings: Vec<ContentFinding>,
}

fn load_schema_artifact(
    project_root: &Path,
    artifact_path: &Path,
    findings: &mut Vec<ContentFinding>,
) -> Option<Value> {
    let absolute = project_root.join(artifact_path);
    let content = match fs::read_to_string(&absolute) {
        Ok(content) => content,
        Err(error) => {
            findings.push(ContentFinding::new(
                "content_schema_read_error",
                Some(artifact_path.to_path_buf()),
                format!(
                    "Failed to read content runtime schema '{}': {error}",
                    artifact_path.display()
                ),
            ));
            return None;
        }
    };
    match serde_json::from_str(&content) {
        Ok(value) => Some(value),
        Err(error) => {
            findings.push(ContentFinding::new(
                "content_schema_parse_error",
                Some(artifact_path.to_path_buf()),
                format!(
                    "Failed to parse content runtime schema '{}': {error}",
                    artifact_path.display()
                ),
            ));
            None
        }
    }
}

fn references_by_collection(
    config: &Config,
    findings: &mut Vec<ContentFinding>,
) -> BTreeMap<String, Vec<ReferenceSpec>> {
    let mut references = BTreeMap::<String, Vec<ReferenceSpec>>::new();
    for (key, relation) in &config.relations {
        let Some((source_collection, field)) = key.split_once('.') else {
            findings.push(ContentFinding::new(
                "invalid_content_relation",
                None,
                format!("Content relation key '{key}' must use collection.field syntax"),
            ));
            continue;
        };
        if source_collection.is_empty() || field.is_empty() || field.contains('.') {
            findings.push(ContentFinding::new(
                "invalid_content_relation",
                None,
                format!("Content relation key '{key}' must use collection.field syntax"),
            ));
            continue;
        }
        if !config.collections.contains_key(source_collection) {
            findings.push(ContentFinding::new(
                "unknown_content_relation_source",
                None,
                format!("Content relation '{key}' references unknown source collection"),
            ));
            continue;
        }
        if relation.target.is_some() && !relation.targets.is_empty() {
            findings.push(ContentFinding::new(
                "invalid_content_relation",
                None,
                format!("Content relation '{key}' must use either target or targets, not both"),
            ));
            continue;
        }
        let mut target_collections = Vec::new();
        if let Some(target) = relation.target.as_ref() {
            target_collections.push(target.clone());
        }
        target_collections.extend(relation.targets.iter().cloned());
        target_collections.sort();
        target_collections.dedup();
        let mut relation_has_error = false;
        for target in &target_collections {
            if !config.collections.contains_key(target) {
                relation_has_error = true;
                findings.push(ContentFinding::new(
                    "unknown_content_relation_target",
                    None,
                    format!(
                        "Content relation '{key}' references unknown target collection '{target}'"
                    ),
                ));
            }
        }
        if relation_has_error {
            continue;
        }
        if target_collections.is_empty() && relation.acyclic {
            findings.push(ContentFinding::new(
                "invalid_content_relation",
                None,
                format!(
                    "Content relation '{key}' must declare target or targets when acyclic is true"
                ),
            ));
            continue;
        }
        references
            .entry(source_collection.to_string())
            .or_default()
            .push(ReferenceSpec {
                field: field.to_string(),
                target_collections,
                many: relation.many,
                required: relation.required,
                acyclic: relation.acyclic,
            });
    }
    for relations in references.values_mut() {
        relations.sort_by(|left, right| left.field.cmp(&right.field));
    }
    references
}

fn literal_pattern_prefix(pattern: &str) -> PathBuf {
    let normalized = pattern.replace('\\', "/");
    let prefix = normalized
        .split(['*', '?', '['])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    normalize_rel_path(Path::new(prefix).to_path_buf())
}

fn normalize_rel_path(path: PathBuf) -> PathBuf {
    if path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        path
    }
}

fn project_relative_path(
    value: &str,
    field: &str,
    code: &'static str,
    findings: &mut Vec<ContentFinding>,
) -> Option<PathBuf> {
    let path = Path::new(value);
    let invalid = value.trim().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)));
    if invalid {
        findings.push(ContentFinding::new(
            code,
            None,
            format!("{field} must be a non-empty project-relative path"),
        ));
        None
    } else {
        Some(normalize_rel_path(path.to_path_buf()))
    }
}
