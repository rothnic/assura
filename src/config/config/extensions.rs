//! Experimental extension configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Experimental extension configuration for first-party custom constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtensionConfig {
    /// First-party custom constraints executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_constraints: Vec<CustomConstraintConfig>,

    /// Configured release artifact contracts executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub release_contracts: Vec<ReleaseContractConfig>,

    /// Configured public surface support matrices executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub support_matrices: Vec<SupportMatrixConfig>,

    /// Configured Cargo manifest semantic policies executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manifest_semantics: Vec<ManifestSemanticsConfig>,

    /// Configured source/test relationship policies executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_relationships: Vec<TestRelationshipConfig>,

    /// Configured Rust module topology policies executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_topologies: Vec<ModuleTopologyConfig>,

    /// Configured docs lifecycle and stale-claim policies executed by
    /// `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub docs_lifecycles: Vec<DocsLifecycleConfig>,

    /// Internal relationship constraints normalized from structure notation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipConstraintConfig>,
}

/// A reusable docs lifecycle and stale-claim policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DocsLifecycleConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Active documentation files checked by this policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active: Vec<String>,
    /// Historical or archived documentation files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub historical: Vec<String>,
    /// Documentation files that must declare an allowed frontmatter status.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub require_frontmatter_status: Vec<String>,
    /// Allowed lifecycle status vocabulary for configured frontmatter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_statuses: Vec<String>,
    /// Deterministic claim token policies checked in active docs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claim_patterns: Vec<DocsLifecycleClaimPatternConfig>,
    /// Historical targets that active docs may reference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub historical_exceptions: Vec<String>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One deterministic stale-claim token and its evidence files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DocsLifecycleClaimPatternConfig {
    /// Stable claim identifier used in diagnostics.
    pub id: String,
    /// Literal token or glob-style token pattern to find in active docs.
    pub pattern: String,
    /// Files expected to carry current evidence for this claim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_files: Vec<String>,
}

/// A reusable Rust module topology policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModuleTopologyConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Module families intentionally classified by this policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<ModuleTopologyModuleConfig>,
    /// Rust source files whose public module/export families must be classified.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rust_exports: Vec<String>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One classified Rust module family.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModuleTopologyModuleConfig {
    /// Module family name, usually the top-level Rust module/export name.
    pub family: String,
    /// Support/topology status for this family.
    pub status: String,
    /// Owning surface, team, or maintainer group.
    pub owner: String,
    /// Short purpose for keeping this module family.
    pub purpose: String,
    /// Root files or directories that must exist for this module family.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roots: Vec<String>,
    /// Public export names intentionally allowed for this family.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_exports: Vec<String>,
    /// Optional visibility marker: `public` or `internal`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
}

/// A reusable source/test evidence policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestRelationshipConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Source-to-test evidence relationships checked by this policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<TestRelationshipSourceConfig>,
    /// Fixture roots whose direct child families must be declared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixture_roots: Vec<String>,
    /// Fixture families accepted under the configured roots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixture_families: Vec<TestRelationshipFixtureFamilyConfig>,
    /// Allowed reason categories for configured ignored/manual tests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_ignore_reasons: Vec<String>,
    /// Ignored/manual tests accepted by this policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_tests: Vec<TestRelationshipIgnoredTestConfig>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One configured source-to-test evidence relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestRelationshipSourceConfig {
    /// Source glob, relative to the project root.
    pub source: String,
    /// Test evidence globs that must match at least one file when source files
    /// exist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_tests: Vec<String>,
}

/// One declared fixture family under a configured fixture root.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestRelationshipFixtureFamilyConfig {
    /// Fixture family directory path relative to the project root.
    pub path: String,
    /// Owning surface or team for the fixture family.
    pub owner: String,
    /// Short purpose for keeping the fixture family.
    pub purpose: String,
}

/// One accepted ignored/manual test file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestRelationshipIgnoredTestConfig {
    /// File path or glob relative to the project root.
    pub path: String,
    /// Ignored test function name accepted by this entry.
    pub test: String,
    /// Reason category, constrained by `allowed_ignore_reasons`.
    pub reason: String,
}

/// A reusable Cargo manifest metadata policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ManifestSemanticsConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Manifest files and package policies checked by this rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manifests: Vec<ManifestSemanticsManifestConfig>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One configured Cargo manifest policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ManifestSemanticsManifestConfig {
    /// Cargo manifest path relative to the project root.
    pub path: String,
    /// Expected package name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Declared role for policy and diagnostics, such as `public` or
    /// `internal`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Expected package version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Expected package rust-version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    /// Expected package license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Expected publish policy: `public` or `internal`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<String>,
    /// Terms that must appear in the package description.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub description_required_terms: Vec<String>,
    /// Terms that must not appear in the package description.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub description_forbidden_terms: Vec<String>,
    /// Keywords that must be declared in package metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    /// Binary names that must be declared in `[[bin]]` entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binaries: Vec<String>,
}

/// A reusable public surface support classification matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SupportMatrixConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Surfaces intentionally classified by this matrix.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<SupportMatrixEntryConfig>,
    /// Command-surface contract files whose command families must be classified.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command_contracts: Vec<String>,
    /// Rust source files whose public export families must be classified.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rust_exports: Vec<String>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One classified public surface entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SupportMatrixEntryConfig {
    /// Surface identifier, such as `command:assura check` or `rust:cli`.
    pub surface: String,
    /// Support status for the surface.
    pub status: String,
}

/// A reusable release artifact synchronization contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReleaseContractConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Artifacts this project intentionally publishes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ReleaseArtifactConfig>,
    /// Workflow files expected to publish the configured artifacts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflow_files: Vec<String>,
    /// Documentation files expected to mention supported artifacts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub docs_files: Vec<String>,
    /// Installer or bootstrap scripts expected to mention supported artifacts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub installer_files: Vec<String>,
    /// Allowed branch names in raw/blob install URLs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_url_branches: Vec<String>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One artifact declared by a release contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReleaseArtifactConfig {
    /// Published archive or package asset name.
    pub name: String,
    /// Whether the artifact requires a `<name>.sha256` sidecar mention.
    #[serde(default)]
    pub checksum_sidecar: bool,
}

/// A first-party custom constraint declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CustomConstraintConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Constraint implementation name.
    #[serde(rename = "type")]
    pub kind: String,
    /// Source glob, relative to the project root.
    pub source: String,
    /// Target path template, relative to the project root.
    pub target: String,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// A capture-based relationship constraint normalized from structure notation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelationshipConstraintConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Source path pattern with named captures, relative to the project root.
    pub source: String,
    /// Structure entry that declared the source side of this relationship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_declaration: Option<String>,
    /// Logical relationship name, such as `doc` or a generated counterpart id.
    pub need: String,
    /// Provider alternatives that can satisfy the need.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<RelationshipProviderConfig>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One provider alternative for a relationship need.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelationshipProviderConfig {
    /// Provider path template with named captures, relative to the project root.
    pub path: String,
    /// Optional Markdown heading text template inside the provider path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Human-readable provider kind used in diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Structure entry that declared this provider alternative.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<String>,
}

/// Checked command-surface contract loaded by the `command_surface_docs`
/// custom constraint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandSurfaceContract {
    /// Supported command families and their documented flag/value surface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<CommandSurfaceCommand>,
}

/// A command family in a checked command-surface contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandSurfaceCommand {
    /// Canonical command name, such as `assura check`.
    pub name: String,
    /// Whether non-flag positional arguments are allowed.
    #[serde(default)]
    pub allow_positionals: bool,
    /// Supported flags keyed by their canonical spelling.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub flags: HashMap<String, CommandSurfaceFlag>,
}

/// A supported flag in a checked command-surface contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandSurfaceFlag {
    /// Whether this flag takes a value.
    #[serde(default)]
    pub takes_value: bool,
    /// Optional aliases such as short flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Optional allowlist of accepted values for value-taking flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    /// Required companion flag values keyed by canonical flag name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub requires: HashMap<String, String>,
}

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
