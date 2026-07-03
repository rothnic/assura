use super::{
    AgentGuidanceConfig, CustomConstraintConfig, DocsLifecycleConfig, ExtensionConfig,
    ManifestSemanticsConfig, ModuleTopologyConfig, RelationshipConstraintConfig,
    ReleaseContractConfig, SupportMatrixConfig, TestRelationshipConfig,
};

impl ExtensionConfig {
    /// Create an empty extension config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom constraint declaration.
    pub fn with_custom_constraint(mut self, constraint: CustomConstraintConfig) -> Self {
        self.custom_constraints.push(constraint);
        self
    }

    /// Add a release artifact contract.
    pub fn with_release_contract(mut self, contract: ReleaseContractConfig) -> Self {
        self.release_contracts.push(contract);
        self
    }

    /// Add a public surface support matrix.
    pub fn with_support_matrix(mut self, matrix: SupportMatrixConfig) -> Self {
        self.support_matrices.push(matrix);
        self
    }

    /// Add a Cargo manifest semantic policy.
    pub fn with_manifest_semantics(mut self, policy: ManifestSemanticsConfig) -> Self {
        self.manifest_semantics.push(policy);
        self
    }

    /// Add a source/test relationship policy.
    pub fn with_test_relationship(mut self, policy: TestRelationshipConfig) -> Self {
        self.test_relationships.push(policy);
        self
    }

    /// Add a Rust module topology policy.
    pub fn with_module_topology(mut self, policy: ModuleTopologyConfig) -> Self {
        self.module_topologies.push(policy);
        self
    }

    /// Add a docs lifecycle and stale-claim policy.
    pub fn with_docs_lifecycle(mut self, policy: DocsLifecycleConfig) -> Self {
        self.docs_lifecycles.push(policy);
        self
    }

    /// Add an agent guidance and skill contract policy.
    pub fn with_agent_guidance(mut self, policy: AgentGuidanceConfig) -> Self {
        self.agent_guidance.push(policy);
        self
    }

    /// Add an internal relationship constraint declaration.
    pub fn with_relationship(mut self, relationship: RelationshipConstraintConfig) -> Self {
        self.relationships.push(relationship);
        self
    }
}

impl CustomConstraintConfig {
    /// Create a paired-file custom constraint.
    pub fn paired_file_exists(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: "paired_file_exists".to_string(),
            source: source.into(),
            target: target.into(),
            severity: None,
        }
    }

    /// Set diagnostic severity.
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = Some(severity.into());
        self
    }
}
