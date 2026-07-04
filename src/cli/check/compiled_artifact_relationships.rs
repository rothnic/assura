use crate::config::config::{RelationshipConstraintConfig, RelationshipProviderConfig};

/// Binary-safe relationship policy stored in compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableRelationshipConstraintConfig {
    id: String,
    source: String,
    source_declaration: Option<String>,
    need: String,
    providers: Vec<PortableRelationshipProviderConfig>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableRelationshipProviderConfig {
    path: String,
    section: Option<String>,
    kind: Option<String>,
    declaration: Option<String>,
}

impl From<RelationshipConstraintConfig> for PortableRelationshipConstraintConfig {
    fn from(config: RelationshipConstraintConfig) -> Self {
        Self {
            id: config.id,
            source: config.source,
            source_declaration: config.source_declaration,
            need: config.need,
            providers: config.providers.into_iter().map(Into::into).collect(),
            severity: config.severity,
        }
    }
}

impl From<PortableRelationshipConstraintConfig> for RelationshipConstraintConfig {
    fn from(config: PortableRelationshipConstraintConfig) -> Self {
        Self {
            id: config.id,
            source: config.source,
            source_declaration: config.source_declaration,
            need: config.need,
            providers: config.providers.into_iter().map(Into::into).collect(),
            severity: config.severity,
        }
    }
}

impl From<RelationshipProviderConfig> for PortableRelationshipProviderConfig {
    fn from(config: RelationshipProviderConfig) -> Self {
        Self {
            path: config.path,
            section: config.section,
            kind: config.kind,
            declaration: config.declaration,
        }
    }
}

impl From<PortableRelationshipProviderConfig> for RelationshipProviderConfig {
    fn from(config: PortableRelationshipProviderConfig) -> Self {
        Self {
            path: config.path,
            section: config.section,
            kind: config.kind,
            declaration: config.declaration,
        }
    }
}
