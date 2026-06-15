/// Binary-safe extension config stored inside compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableExtensionConfig {
    custom_constraints: Vec<PortableCustomConstraintConfig>,
    relationships: Vec<PortableRelationshipConstraintConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableCustomConstraintConfig {
    id: String,
    kind: String,
    source: String,
    target: String,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableRelationshipConstraintConfig {
    id: String,
    source: String,
    need: String,
    providers: Vec<PortableRelationshipProviderConfig>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableRelationshipProviderConfig {
    path: String,
    section: Option<String>,
}

impl From<ExtensionConfig> for PortableExtensionConfig {
    fn from(config: ExtensionConfig) -> Self {
        Self {
            custom_constraints: config
                .custom_constraints
                .into_iter()
                .map(Into::into)
                .collect(),
            relationships: config.relationships.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PortableExtensionConfig> for ExtensionConfig {
    fn from(config: PortableExtensionConfig) -> Self {
        Self {
            custom_constraints: config
                .custom_constraints
                .into_iter()
                .map(Into::into)
                .collect(),
            relationships: config.relationships.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CustomConstraintConfig> for PortableCustomConstraintConfig {
    fn from(config: CustomConstraintConfig) -> Self {
        Self {
            id: config.id,
            kind: config.kind,
            source: config.source,
            target: config.target,
            severity: config.severity,
        }
    }
}

impl From<PortableCustomConstraintConfig> for CustomConstraintConfig {
    fn from(config: PortableCustomConstraintConfig) -> Self {
        Self {
            id: config.id,
            kind: config.kind,
            source: config.source,
            target: config.target,
            severity: config.severity,
        }
    }
}

impl From<RelationshipConstraintConfig> for PortableRelationshipConstraintConfig {
    fn from(config: RelationshipConstraintConfig) -> Self {
        Self {
            id: config.id,
            source: config.source,
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
        }
    }
}

impl From<PortableRelationshipProviderConfig> for RelationshipProviderConfig {
    fn from(config: PortableRelationshipProviderConfig) -> Self {
        Self {
            path: config.path,
            section: config.section,
        }
    }
}
