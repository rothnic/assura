//! Content-runtime activation checks for project doctor output.

use super::doctor::DoctorItem;
use crate::config::config::Config;
use crate::content_repository::{ContentFinding, ContentRepository, RepositoryModel};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub(super) fn content_runtime_gaps(project_root: &Path, config: &Config) -> Vec<DoctorItem> {
    if config.models.is_none() && config.collections.is_empty() {
        return Vec::new();
    }
    let state = ContentDoctorState::load(project_root, config);
    let mut gaps = Vec::new();
    for finding in &state.findings {
        if matches!(
            finding.code,
            "content_schema_read_error"
                | "content_schema_parse_error"
                | "content_schema_compile_error"
                | "content_schema_missing"
        ) {
            gaps.push(DoctorItem {
                name: "content_schema_unavailable".to_string(),
                status: "gap",
                detail: finding.message.clone(),
            });
        }
    }
    for collection in config.collections.keys() {
        let count = state
            .collection_counts
            .get(collection)
            .copied()
            .unwrap_or(0);
        if count == 0 {
            gaps.push(DoctorItem {
                name: format!("empty_collection:{collection}"),
                status: "gap",
                detail: format!(
                    "Configured content collection `{collection}` has no indexed record files."
                ),
            });
        }
    }
    if !config.collections.is_empty() && state.object_count == 0 && state.findings.is_empty() {
        gaps.push(DoctorItem {
            name: "zero_search_chunks".to_string(),
            status: "gap",
            detail: "Active content runtime has no indexed model records, so modeled search has zero chunks.".to_string(),
        });
    }
    for key in config.relations.keys() {
        let Some((collection, field)) = key.split_once('.') else {
            continue;
        };
        let edge_key = format!("{collection}.{field}");
        if !state.relation_edges.contains(&edge_key) {
            gaps.push(DoctorItem {
                name: format!("relation_without_edges:{edge_key}"),
                status: "gap",
                detail: format!("Configured content relation `{edge_key}` produced no edges."),
            });
        }
    }
    gaps
}

#[derive(Default)]
struct ContentDoctorState {
    collection_counts: BTreeMap<String, usize>,
    relation_edges: HashSet<String>,
    object_count: usize,
    findings: Vec<ContentFinding>,
}

impl ContentDoctorState {
    fn load(project_root: &Path, config: &Config) -> Self {
        let model = match RepositoryModel::from_config(project_root, config) {
            Ok(model) => model,
            Err(findings) => {
                return Self {
                    findings,
                    ..Self::default()
                };
            }
        };
        let repository = match ContentRepository::try_new(model) {
            Ok(repository) => repository,
            Err(findings) => {
                return Self {
                    findings,
                    ..Self::default()
                };
            }
        };
        let validation = repository.validate(project_root);
        let mut collection_counts = BTreeMap::new();
        for (collection, _) in validation.snapshot.objects.keys() {
            *collection_counts.entry(collection.clone()).or_default() += 1;
        }
        let relation_edges = validation
            .snapshot
            .edges
            .iter()
            .map(|edge| format!("{}.{}", edge.source.collection, edge.field))
            .collect::<HashSet<_>>();
        Self {
            object_count: validation.snapshot.objects.len(),
            collection_counts,
            relation_edges,
            findings: validation.findings,
        }
    }
}
