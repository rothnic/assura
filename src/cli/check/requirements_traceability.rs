//! Requirements, claims, evidence, and finding traceability validation.

use super::{StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::RequirementsTraceabilityConfig;
use crate::content_repository::{
    ContentFinding, ContentRepository, RepoEdge, RepoObject, RepositoryValidation,
};
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

impl StructureChecker {
    pub(super) fn validate_requirements_traceability(
        &self,
        policies: &[RequirementsTraceabilityConfig],
        report: &mut StructureCheckReport,
    ) {
        if policies.is_empty() {
            return;
        }

        let validation = match ContentRepository::from_config(&self.project_root, &self.config) {
            Ok(repository) => repository.validate(&self.project_root),
            Err(findings) => {
                for policy in policies {
                    for finding in &findings {
                        self.push_traceability_runtime_violation(report, policy, finding);
                    }
                }
                return;
            }
        };

        for policy in policies {
            self.validate_traceability_policy(policy, &validation, report);
        }
    }

    fn validate_traceability_policy(
        &self,
        policy: &RequirementsTraceabilityConfig,
        validation: &RepositoryValidation,
        report: &mut StructureCheckReport,
    ) {
        let collections = collection_names(validation);
        self.validate_policy_collections(policy, &collections, report);
        self.validate_high_priority_requirement_coverage(policy, validation, report);
        self.validate_claim_evidence_links(policy, validation, report);
        self.validate_evidence_source_links(policy, validation, report);
        self.validate_finding_metadata(policy, validation, report);
    }

    fn validate_policy_collections(
        &self,
        policy: &RequirementsTraceabilityConfig,
        collections: &HashSet<String>,
        report: &mut StructureCheckReport,
    ) {
        let config_path = self.relative_path(&report.config_path);
        for collection in configured_policy_collections(policy) {
            if collections.contains(&collection) {
                continue;
            }
            self.push_traceability_violation(
                report,
                policy,
                config_path.clone(),
                format!(
                    "Requirements traceability `{}` references missing content collection `{collection}`",
                    policy.id
                ),
            );
        }
    }

    fn validate_high_priority_requirement_coverage(
        &self,
        policy: &RequirementsTraceabilityConfig,
        validation: &RepositoryValidation,
        report: &mut StructureCheckReport,
    ) {
        if policy.high_priority_values.is_empty() {
            return;
        }
        let high_priority = policy
            .high_priority_values
            .iter()
            .map(|value| value.to_ascii_lowercase())
            .collect::<HashSet<_>>();
        let coverage_collections = policy
            .coverage_collections
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let incoming_coverage = incoming_targets_from_sources(
            validation,
            &policy.requirements_collection,
            &coverage_collections,
        );

        for requirement in objects_in_collection(validation, &policy.requirements_collection) {
            let Some(priority) = string_field(requirement, &policy.priority_field) else {
                continue;
            };
            if !high_priority.contains(&priority.to_ascii_lowercase()) {
                continue;
            }
            if incoming_coverage.contains(&requirement.id) {
                continue;
            }
            self.push_traceability_violation(
                report,
                policy,
                requirement.rel_path.clone(),
                format!(
                    "High-priority requirement `{}` has no coverage from configured collections: {}",
                    requirement.id,
                    policy.coverage_collections.join(", ")
                ),
            );
        }
    }

    fn validate_claim_evidence_links(
        &self,
        policy: &RequirementsTraceabilityConfig,
        validation: &RepositoryValidation,
        report: &mut StructureCheckReport,
    ) {
        if policy.claim_collections.is_empty() {
            return;
        }
        let evidence_targets = policy
            .evidence_collections
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        for claim_collection in &policy.claim_collections {
            for claim in objects_in_collection(validation, claim_collection) {
                if has_outgoing_target(validation, claim, &evidence_targets) {
                    continue;
                }
                self.push_traceability_violation(
                    report,
                    policy,
                    claim.rel_path.clone(),
                    format!(
                        "Claim `{}` must link to evidence through a modeled relation",
                        claim.id
                    ),
                );
            }
        }
    }

    fn validate_evidence_source_links(
        &self,
        policy: &RequirementsTraceabilityConfig,
        validation: &RepositoryValidation,
        report: &mut StructureCheckReport,
    ) {
        if policy.evidence_collections.is_empty() {
            return;
        }
        let source_targets = policy
            .source_document_collections
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        for evidence_collection in &policy.evidence_collections {
            for evidence in objects_in_collection(validation, evidence_collection) {
                if has_outgoing_target(validation, evidence, &source_targets) {
                    continue;
                }
                self.push_traceability_violation(
                    report,
                    policy,
                    evidence.rel_path.clone(),
                    format!(
                        "Evidence `{}` must link to a source document through a modeled relation",
                        evidence.id
                    ),
                );
            }
        }
    }

    fn validate_finding_metadata(
        &self,
        policy: &RequirementsTraceabilityConfig,
        validation: &RepositoryValidation,
        report: &mut StructureCheckReport,
    ) {
        if policy.finding_collections.is_empty() {
            return;
        }
        for finding_collection in &policy.finding_collections {
            for finding in objects_in_collection(validation, finding_collection) {
                if !has_any_non_empty_field(finding, &policy.owner_fields) {
                    self.push_traceability_violation(
                        report,
                        policy,
                        finding.rel_path.clone(),
                        format!(
                            "Finding `{}` must carry owner metadata in one of: {}",
                            finding.id,
                            policy.owner_fields.join(", ")
                        ),
                    );
                }
                if !has_any_non_empty_field(finding, &policy.status_fields) {
                    self.push_traceability_violation(
                        report,
                        policy,
                        finding.rel_path.clone(),
                        format!(
                            "Finding `{}` must carry status metadata in one of: {}",
                            finding.id,
                            policy.status_fields.join(", ")
                        ),
                    );
                }
            }
        }
    }

    fn push_traceability_runtime_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &RequirementsTraceabilityConfig,
        finding: &ContentFinding,
    ) {
        let path = finding
            .path
            .clone()
            .unwrap_or_else(|| self.relative_path(&report.config_path));
        self.push_traceability_violation(
            report,
            policy,
            path,
            format!(
                "Requirements traceability `{}` could not load content runtime: {}",
                policy.id, finding.message
            ),
        );
    }

    fn push_traceability_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &RequirementsTraceabilityConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("requirements_traceability:{}", policy.id),
            message,
            policy.severity.as_deref().unwrap_or("medium"),
        ));
    }
}

fn collection_names(validation: &RepositoryValidation) -> HashSet<String> {
    validation
        .snapshot
        .objects
        .keys()
        .map(|(collection, _)| collection.clone())
        .collect()
}

fn configured_policy_collections(policy: &RequirementsTraceabilityConfig) -> HashSet<String> {
    let mut collections = HashSet::new();
    collections.insert(policy.requirements_collection.clone());
    collections.extend(policy.coverage_collections.iter().cloned());
    collections.extend(policy.claim_collections.iter().cloned());
    collections.extend(policy.evidence_collections.iter().cloned());
    collections.extend(policy.source_document_collections.iter().cloned());
    collections.extend(policy.finding_collections.iter().cloned());
    collections
}

fn objects_in_collection<'a>(
    validation: &'a RepositoryValidation,
    collection: &str,
) -> Vec<&'a RepoObject> {
    validation
        .snapshot
        .objects
        .iter()
        .filter_map(|((object_collection, _), object)| {
            (object_collection == collection).then_some(object)
        })
        .collect()
}

fn incoming_targets_from_sources(
    validation: &RepositoryValidation,
    target_collection: &str,
    source_collections: &HashSet<&str>,
) -> HashSet<String> {
    validation
        .snapshot
        .edges
        .iter()
        .filter(|edge| {
            source_collections.contains(edge.source.collection.as_str())
                && edge
                    .target_collections
                    .iter()
                    .any(|collection| collection == target_collection)
                && edge_resolves(validation, edge, target_collection)
        })
        .map(|edge| edge.target_id.clone())
        .collect()
}

fn has_outgoing_target(
    validation: &RepositoryValidation,
    source: &RepoObject,
    target_collections: &HashSet<&str>,
) -> bool {
    validation.snapshot.edges.iter().any(|edge| {
        edge.source.collection == source.collection
            && edge.source.id == source.id
            && edge
                .target_collections
                .iter()
                .any(|collection| target_collections.contains(collection.as_str()))
            && edge.target_collections.iter().any(|collection| {
                target_collections.contains(collection.as_str())
                    && edge_resolves(validation, edge, collection)
            })
    })
}

fn edge_resolves(
    validation: &RepositoryValidation,
    edge: &RepoEdge,
    target_collection: &str,
) -> bool {
    validation
        .snapshot
        .objects
        .contains_key(&(target_collection.to_string(), edge.target_id.clone()))
}

fn string_field<'a>(object: &'a RepoObject, field: &str) -> Option<&'a str> {
    object.data.get(field).and_then(Value::as_str)
}

fn has_any_non_empty_field(object: &RepoObject, fields: &[String]) -> bool {
    fields
        .iter()
        .any(|field| non_empty_value(object.data.get(field)))
}

fn non_empty_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(value)) => !value.trim().is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(items)) => !items.is_empty(),
        Some(Value::Bool(_)) | Some(Value::Number(_)) => true,
        Some(Value::Null) | None => false,
    }
}
