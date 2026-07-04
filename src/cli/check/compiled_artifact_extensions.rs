use crate::config::config::{
    CustomConstraintConfig, DocsLifecycleClaimPatternConfig, DocsLifecycleConfig, ExtensionConfig,
    ManifestSemanticsConfig, ManifestSemanticsManifestConfig, ModuleTopologyConfig,
    ReleaseArtifactConfig, ReleaseContractConfig, RepositoryReferenceConfig, SupportMatrixConfig,
    SupportMatrixDocsClaimSourceConfig, SupportMatrixEntryConfig, TestRelationshipConfig,
    TestRelationshipFixtureFamilyConfig, TestRelationshipIgnoredTestConfig, TestRelationshipSourceConfig,
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
    docs_lifecycles: Vec<PortableDocsLifecycleConfig>,
    repository_references: Vec<RepositoryReferenceConfig>,
    #[serde(default)]
    agent_guidance: Vec<PortableAgentGuidanceConfig>,
    #[serde(default)]
    requirements_traceability: Vec<PortableRequirementsTraceabilityConfig>,
    #[serde(default)]
    computed_checks: Vec<PortableComputedCheckConfig>,
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
    entries: Vec<SupportMatrixEntryConfig>,
    command_contracts: Vec<String>,
    rust_exports: Vec<String>,
    docs_claim_sources: Vec<SupportMatrixDocsClaimSourceConfig>,
    manifest_policies: Vec<String>,
    severity: Option<String>,
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
            docs_lifecycles: config.docs_lifecycles.into_iter().map(Into::into).collect(),
            repository_references: config.repository_references,
            agent_guidance: config.agent_guidance.into_iter().map(Into::into).collect(),
            requirements_traceability: config
                .requirements_traceability
                .into_iter()
                .map(Into::into)
                .collect(),
            computed_checks: config.computed_checks.into_iter().map(Into::into).collect(),
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
            docs_lifecycles: config.docs_lifecycles.into_iter().map(Into::into).collect(),
            repository_references: config.repository_references,
            agent_guidance: config.agent_guidance.into_iter().map(Into::into).collect(),
            requirements_traceability: config
                .requirements_traceability
                .into_iter()
                .map(Into::into)
                .collect(),
            computed_checks: config.computed_checks.into_iter().map(Into::into).collect(),
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
            entries: config.entries,
            command_contracts: config.command_contracts,
            rust_exports: config.rust_exports,
            docs_claim_sources: config.docs_claim_sources,
            manifest_policies: config.manifest_policies,
            severity: config.severity,
        }
    }
}

impl From<PortableSupportMatrixConfig> for SupportMatrixConfig {
    fn from(config: PortableSupportMatrixConfig) -> Self {
        Self {
            id: config.id,
            entries: config.entries,
            command_contracts: config.command_contracts,
            rust_exports: config.rust_exports,
            docs_claim_sources: config.docs_claim_sources,
            manifest_policies: config.manifest_policies,
            severity: config.severity,
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
