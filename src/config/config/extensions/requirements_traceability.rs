use serde::{Deserialize, Serialize};

/// Reusable content-runtime traceability policy for requirements, claims,
/// evidence, source documents, and findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RequirementsTraceabilityConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Content collection containing Requirement records.
    pub requirements_collection: String,
    /// Requirement field that stores priority labels.
    pub priority_field: String,
    /// Priority values treated as coverage-required.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub high_priority_values: Vec<String>,
    /// Collections that may cover requirements through modeled relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coverage_collections: Vec<String>,
    /// Collections containing Claim records that must link to evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claim_collections: Vec<String>,
    /// Collections containing Evidence records that must link to source docs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_collections: Vec<String>,
    /// Collections containing SourceDocument records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_document_collections: Vec<String>,
    /// Collections containing Finding records that need metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finding_collections: Vec<String>,
    /// Finding fields accepted as owner metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_fields: Vec<String>,
    /// Finding fields accepted as status metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status_fields: Vec<String>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}
