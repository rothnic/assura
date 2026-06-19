use crate::config::config::{
    CustomConstraintConfig, ExtensionConfig, RelationshipConstraintConfig,
    RelationshipProviderConfig, ReleaseArtifactConfig, ReleaseContractConfig, SupportMatrixConfig,
    SupportMatrixEntryConfig,
};

/// Binary-safe extension config stored inside compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableExtensionConfig {
    custom_constraints: Vec<PortableCustomConstraintConfig>,
    release_contracts: Vec<PortableReleaseContractConfig>,
    support_matrices: Vec<PortableSupportMatrixConfig>,
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
struct PortableReleaseContractConfig {
    id: String,
    artifacts: Vec<PortableReleaseArtifactConfig>,
    workflow_files: Vec<String>,
    docs_files: Vec<String>,
    installer_files: Vec<String>,
    allowed_url_branches: Vec<String>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableReleaseArtifactConfig {
    name: String,
    checksum_sidecar: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableSupportMatrixConfig {
    id: String,
    entries: Vec<PortableSupportMatrixEntryConfig>,
    command_contracts: Vec<String>,
    rust_exports: Vec<String>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableSupportMatrixEntryConfig {
    surface: String,
    status: String,
}

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

impl From<ExtensionConfig> for PortableExtensionConfig {
    fn from(config: ExtensionConfig) -> Self {
        Self {
            custom_constraints: config
                .custom_constraints
                .into_iter()
                .map(Into::into)
                .collect(),
            release_contracts: config
                .release_contracts
                .into_iter()
                .map(Into::into)
                .collect(),
            support_matrices: config
                .support_matrices
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
            release_contracts: config
                .release_contracts
                .into_iter()
                .map(Into::into)
                .collect(),
            support_matrices: config
                .support_matrices
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

impl From<ReleaseContractConfig> for PortableReleaseContractConfig {
    fn from(config: ReleaseContractConfig) -> Self {
        Self {
            id: config.id,
            artifacts: config.artifacts.into_iter().map(Into::into).collect(),
            workflow_files: config.workflow_files,
            docs_files: config.docs_files,
            installer_files: config.installer_files,
            allowed_url_branches: config.allowed_url_branches,
            severity: config.severity,
        }
    }
}

impl From<PortableReleaseContractConfig> for ReleaseContractConfig {
    fn from(config: PortableReleaseContractConfig) -> Self {
        Self {
            id: config.id,
            artifacts: config.artifacts.into_iter().map(Into::into).collect(),
            workflow_files: config.workflow_files,
            docs_files: config.docs_files,
            installer_files: config.installer_files,
            allowed_url_branches: config.allowed_url_branches,
            severity: config.severity,
        }
    }
}

impl From<ReleaseArtifactConfig> for PortableReleaseArtifactConfig {
    fn from(config: ReleaseArtifactConfig) -> Self {
        Self {
            name: config.name,
            checksum_sidecar: config.checksum_sidecar,
        }
    }
}

impl From<PortableReleaseArtifactConfig> for ReleaseArtifactConfig {
    fn from(config: PortableReleaseArtifactConfig) -> Self {
        Self {
            name: config.name,
            checksum_sidecar: config.checksum_sidecar,
        }
    }
}

impl From<SupportMatrixConfig> for PortableSupportMatrixConfig {
    fn from(config: SupportMatrixConfig) -> Self {
        Self {
            id: config.id,
            entries: config.entries.into_iter().map(Into::into).collect(),
            command_contracts: config.command_contracts,
            rust_exports: config.rust_exports,
            severity: config.severity,
        }
    }
}

impl From<PortableSupportMatrixConfig> for SupportMatrixConfig {
    fn from(config: PortableSupportMatrixConfig) -> Self {
        Self {
            id: config.id,
            entries: config.entries.into_iter().map(Into::into).collect(),
            command_contracts: config.command_contracts,
            rust_exports: config.rust_exports,
            severity: config.severity,
        }
    }
}

impl From<SupportMatrixEntryConfig> for PortableSupportMatrixEntryConfig {
    fn from(config: SupportMatrixEntryConfig) -> Self {
        Self {
            surface: config.surface,
            status: config.status,
        }
    }
}

impl From<PortableSupportMatrixEntryConfig> for SupportMatrixEntryConfig {
    fn from(config: PortableSupportMatrixEntryConfig) -> Self {
        Self {
            surface: config.surface,
            status: config.status,
        }
    }
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
