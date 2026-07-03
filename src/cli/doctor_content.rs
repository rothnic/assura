//! Content-runtime activation checks for project doctor output.

use super::doctor::DoctorItem;
use crate::config::config::{Config, RequirementsTraceabilityConfig};
use crate::content_repository::{ContentFinding, ContentRepository, RepositoryModel};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub(super) fn content_runtime_gaps(project_root: &Path, config: &Config) -> Vec<DoctorItem> {
    let mut computed_gaps = computed_check_gaps(project_root, config);
    let traceability_policies = config
        .extensions
        .as_ref()
        .map(|extensions| extensions.requirements_traceability.as_slice())
        .unwrap_or(&[]);
    if config.models.is_none() && config.collections.is_empty() {
        if traceability_policies.is_empty() {
            return computed_gaps;
        }
        computed_gaps.extend(traceability_policies
            .iter()
            .map(|policy| DoctorItem {
                name: format!("requirements_traceability_inactive:{}", policy.id),
                status: "gap",
                detail: format!(
                    "Requirements traceability `{}` is configured but content runtime models and collections are inactive.",
                    policy.id
                ),
            }));
        return computed_gaps;
    }
    if config.models.is_none() && !traceability_policies.is_empty() {
        computed_gaps.extend(traceability_policies
            .iter()
            .map(|policy| DoctorItem {
                name: format!("requirements_traceability_inactive:{}", policy.id),
                status: "gap",
                detail: format!(
                    "Requirements traceability `{}` is configured but models.validation_artifact is missing.",
                    policy.id
                ),
            }));
        return computed_gaps;
    }
    let state = ContentDoctorState::load(project_root, config);
    let mut gaps = computed_gaps;
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
    for policy in traceability_policies {
        gaps.extend(traceability_policy_gaps(policy, config, &state));
    }
    gaps
}

fn computed_check_gaps(project_root: &Path, config: &Config) -> Vec<DoctorItem> {
    let Some(extensions) = &config.extensions else {
        return Vec::new();
    };
    extensions
        .computed_checks
        .iter()
        .filter_map(|policy| {
            let script_path = project_root.join(&policy.script);
            (!script_path.is_file()).then(|| DoctorItem {
                name: format!("computed_check_script_missing:{}", policy.id),
                status: "gap",
                detail: format!(
                    "Computed check `{}` script `{}` is configured but missing.",
                    policy.id, policy.script
                ),
            })
        })
        .collect()
}

fn traceability_policy_gaps(
    policy: &RequirementsTraceabilityConfig,
    config: &Config,
    state: &ContentDoctorState,
) -> Vec<DoctorItem> {
    let mut gaps = Vec::new();
    for collection in traceability_policy_collections(policy) {
        if !config.collections.contains_key(&collection) {
            gaps.push(DoctorItem {
                name: format!("requirements_traceability_missing_collection:{collection}"),
                status: "gap",
                detail: format!(
                    "Requirements traceability `{}` references collection `{collection}`, but it is not configured.",
                    policy.id
                ),
            });
            continue;
        }
        if state
            .collection_counts
            .get(&collection)
            .copied()
            .unwrap_or(0)
            == 0
        {
            gaps.push(DoctorItem {
                name: format!("requirements_traceability_empty_collection:{collection}"),
                status: "gap",
                detail: format!(
                    "Requirements traceability `{}` references collection `{collection}`, but no records were indexed.",
                    policy.id
                ),
            });
        }
    }
    gaps
}

fn traceability_policy_collections(policy: &RequirementsTraceabilityConfig) -> HashSet<String> {
    let mut collections = HashSet::new();
    collections.insert(policy.requirements_collection.clone());
    collections.extend(policy.coverage_collections.iter().cloned());
    collections.extend(policy.claim_collections.iter().cloned());
    collections.extend(policy.evidence_collections.iter().cloned());
    collections.extend(policy.source_document_collections.iter().cloned());
    collections.extend(policy.finding_collections.iter().cloned());
    collections
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
