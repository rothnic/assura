use crate::config::config::{
    CustomConstraintConfig, ExtensionConfig, ManifestSemanticsConfig,
    ManifestSemanticsManifestConfig, RelationshipConstraintConfig, RelationshipProviderConfig,
    ReleaseArtifactConfig, ReleaseContractConfig, SupportMatrixConfig, SupportMatrixEntryConfig,
    TestRelationshipConfig, TestRelationshipFixtureFamilyConfig,
    TestRelationshipIgnoredTestConfig, TestRelationshipSourceConfig, ModuleTopologyConfig,
};

/// Binary-safe extension config stored inside compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableExtensionConfig {
    custom_constraints: Vec<PortableCustomConstraintConfig>,
    release_contracts: Vec<PortableReleaseContractConfig>,
    support_matrices: Vec<PortableSupportMatrixConfig>,
    manifest_semantics: Vec<PortableManifestSemanticsConfig>,
    test_relationships: Vec<PortableTestRelationshipConfig>,
    module_topologies: Vec<PortableModuleTopologyConfig>,
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
struct PortableManifestSemanticsConfig {
    id: String,
    manifests: Vec<PortableManifestSemanticsManifestConfig>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableManifestSemanticsManifestConfig {
    path: String,
    package: Option<String>,
    role: Option<String>,
    version: Option<String>,
    rust_version: Option<String>,
    license: Option<String>,
    publish: Option<String>,
    description_required_terms: Vec<String>,
    description_forbidden_terms: Vec<String>,
    keywords: Vec<String>,
    binaries: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableTestRelationshipConfig {
    id: String,
    relationships: Vec<PortableTestRelationshipSourceConfig>,
    fixture_roots: Vec<String>,
    fixture_families: Vec<PortableTestRelationshipFixtureFamilyConfig>,
    allowed_ignore_reasons: Vec<String>,
    ignored_tests: Vec<PortableTestRelationshipIgnoredTestConfig>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableTestRelationshipSourceConfig {
    source: String,
    required_tests: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableTestRelationshipFixtureFamilyConfig {
    path: String,
    owner: String,
    purpose: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableTestRelationshipIgnoredTestConfig {
    path: String,
    test: String,
    reason: String,
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
            manifest_semantics: config
                .manifest_semantics
                .into_iter()
                .map(Into::into)
                .collect(),
            test_relationships: config.test_relationships.into_iter().map(Into::into).collect(),
            module_topologies: config.module_topologies.into_iter().map(Into::into).collect(),
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
            manifest_semantics: config
                .manifest_semantics
                .into_iter()
                .map(Into::into)
                .collect(),
            test_relationships: config.test_relationships.into_iter().map(Into::into).collect(),
            module_topologies: config.module_topologies.into_iter().map(Into::into).collect(),
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

impl From<ManifestSemanticsConfig> for PortableManifestSemanticsConfig {
    fn from(config: ManifestSemanticsConfig) -> Self {
        Self {
            id: config.id,
            manifests: config.manifests.into_iter().map(Into::into).collect(),
            severity: config.severity,
        }
    }
}

impl From<PortableManifestSemanticsConfig> for ManifestSemanticsConfig {
    fn from(config: PortableManifestSemanticsConfig) -> Self {
        Self {
            id: config.id,
            manifests: config.manifests.into_iter().map(Into::into).collect(),
            severity: config.severity,
        }
    }
}

impl From<ManifestSemanticsManifestConfig> for PortableManifestSemanticsManifestConfig {
    fn from(config: ManifestSemanticsManifestConfig) -> Self {
        Self {
            path: config.path,
            package: config.package,
            role: config.role,
            version: config.version,
            rust_version: config.rust_version,
            license: config.license,
            publish: config.publish,
            description_required_terms: config.description_required_terms,
            description_forbidden_terms: config.description_forbidden_terms,
            keywords: config.keywords,
            binaries: config.binaries,
        }
    }
}

impl From<PortableManifestSemanticsManifestConfig> for ManifestSemanticsManifestConfig {
    fn from(config: PortableManifestSemanticsManifestConfig) -> Self {
        Self {
            path: config.path,
            package: config.package,
            role: config.role,
            version: config.version,
            rust_version: config.rust_version,
            license: config.license,
            publish: config.publish,
            description_required_terms: config.description_required_terms,
            description_forbidden_terms: config.description_forbidden_terms,
            keywords: config.keywords,
            binaries: config.binaries,
        }
    }
}

impl From<TestRelationshipConfig> for PortableTestRelationshipConfig {
    fn from(config: TestRelationshipConfig) -> Self {
        Self {
            id: config.id,
            relationships: config.relationships.into_iter().map(Into::into).collect(),
            fixture_roots: config.fixture_roots,
            fixture_families: config
                .fixture_families
                .into_iter()
                .map(Into::into)
                .collect(),
            allowed_ignore_reasons: config.allowed_ignore_reasons,
            ignored_tests: config.ignored_tests.into_iter().map(Into::into).collect(),
            severity: config.severity,
        }
    }
}
impl From<PortableTestRelationshipConfig> for TestRelationshipConfig {
    fn from(config: PortableTestRelationshipConfig) -> Self {
        Self {
            id: config.id,
            relationships: config.relationships.into_iter().map(Into::into).collect(),
            fixture_roots: config.fixture_roots,
            fixture_families: config
                .fixture_families
                .into_iter()
                .map(Into::into)
                .collect(),
            allowed_ignore_reasons: config.allowed_ignore_reasons,
            ignored_tests: config.ignored_tests.into_iter().map(Into::into).collect(),
            severity: config.severity,
        }
    }
}
impl From<TestRelationshipSourceConfig> for PortableTestRelationshipSourceConfig {
    fn from(config: TestRelationshipSourceConfig) -> Self {
        Self {
            source: config.source,
            required_tests: config.required_tests,
        }
    }
}
impl From<PortableTestRelationshipSourceConfig> for TestRelationshipSourceConfig {
    fn from(config: PortableTestRelationshipSourceConfig) -> Self {
        Self {
            source: config.source,
            required_tests: config.required_tests,
        }
    }
}
impl From<TestRelationshipFixtureFamilyConfig> for PortableTestRelationshipFixtureFamilyConfig {
    fn from(config: TestRelationshipFixtureFamilyConfig) -> Self {
        Self {
            path: config.path,
            owner: config.owner,
            purpose: config.purpose,
        }
    }
}
impl From<PortableTestRelationshipFixtureFamilyConfig> for TestRelationshipFixtureFamilyConfig {
    fn from(config: PortableTestRelationshipFixtureFamilyConfig) -> Self {
        Self {
            path: config.path,
            owner: config.owner,
            purpose: config.purpose,
        }
    }
}
impl From<TestRelationshipIgnoredTestConfig> for PortableTestRelationshipIgnoredTestConfig {
    fn from(config: TestRelationshipIgnoredTestConfig) -> Self {
        Self {
            path: config.path,
            test: config.test,
            reason: config.reason,
        }
    }
}
impl From<PortableTestRelationshipIgnoredTestConfig> for TestRelationshipIgnoredTestConfig {
    fn from(config: PortableTestRelationshipIgnoredTestConfig) -> Self {
        Self {
            path: config.path,
            test: config.test,
            reason: config.reason,
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
