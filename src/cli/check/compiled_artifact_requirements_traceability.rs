use crate::config::config::RequirementsTraceabilityConfig;

/// Binary-safe requirements traceability policy stored in compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableRequirementsTraceabilityConfig {
    id: String,
    requirements_collection: String,
    priority_field: String,
    high_priority_values: Vec<String>,
    coverage_collections: Vec<String>,
    claim_collections: Vec<String>,
    evidence_collections: Vec<String>,
    source_document_collections: Vec<String>,
    finding_collections: Vec<String>,
    owner_fields: Vec<String>,
    status_fields: Vec<String>,
    severity: Option<String>,
}

impl From<RequirementsTraceabilityConfig> for PortableRequirementsTraceabilityConfig {
    fn from(config: RequirementsTraceabilityConfig) -> Self {
        Self {
            id: config.id,
            requirements_collection: config.requirements_collection,
            priority_field: config.priority_field,
            high_priority_values: config.high_priority_values,
            coverage_collections: config.coverage_collections,
            claim_collections: config.claim_collections,
            evidence_collections: config.evidence_collections,
            source_document_collections: config.source_document_collections,
            finding_collections: config.finding_collections,
            owner_fields: config.owner_fields,
            status_fields: config.status_fields,
            severity: config.severity,
        }
    }
}

impl From<PortableRequirementsTraceabilityConfig> for RequirementsTraceabilityConfig {
    fn from(config: PortableRequirementsTraceabilityConfig) -> Self {
        Self {
            id: config.id,
            requirements_collection: config.requirements_collection,
            priority_field: config.priority_field,
            high_priority_values: config.high_priority_values,
            coverage_collections: config.coverage_collections,
            claim_collections: config.claim_collections,
            evidence_collections: config.evidence_collections,
            source_document_collections: config.source_document_collections,
            finding_collections: config.finding_collections,
            owner_fields: config.owner_fields,
            status_fields: config.status_fields,
            severity: config.severity,
        }
    }
}
