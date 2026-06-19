/// Portable docs-lifecycle extension config for compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableDocsLifecycleConfig {
    id: String,
    active: Vec<String>,
    historical: Vec<String>,
    require_frontmatter_status: Vec<String>,
    allowed_statuses: Vec<String>,
    claim_patterns: Vec<PortableDocsLifecycleClaimPatternConfig>,
    historical_exceptions: Vec<String>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableDocsLifecycleClaimPatternConfig {
    id: String,
    pattern: String,
    evidence_files: Vec<String>,
}

impl From<DocsLifecycleConfig> for PortableDocsLifecycleConfig {
    fn from(config: DocsLifecycleConfig) -> Self {
        Self {
            id: config.id,
            active: config.active,
            historical: config.historical,
            require_frontmatter_status: config.require_frontmatter_status,
            allowed_statuses: config.allowed_statuses,
            claim_patterns: config.claim_patterns.into_iter().map(Into::into).collect(),
            historical_exceptions: config.historical_exceptions,
            severity: config.severity,
        }
    }
}

impl From<PortableDocsLifecycleConfig> for DocsLifecycleConfig {
    fn from(config: PortableDocsLifecycleConfig) -> Self {
        Self {
            id: config.id,
            active: config.active,
            historical: config.historical,
            require_frontmatter_status: config.require_frontmatter_status,
            allowed_statuses: config.allowed_statuses,
            claim_patterns: config.claim_patterns.into_iter().map(Into::into).collect(),
            historical_exceptions: config.historical_exceptions,
            severity: config.severity,
        }
    }
}

impl From<DocsLifecycleClaimPatternConfig> for PortableDocsLifecycleClaimPatternConfig {
    fn from(config: DocsLifecycleClaimPatternConfig) -> Self {
        Self {
            id: config.id,
            pattern: config.pattern,
            evidence_files: config.evidence_files,
        }
    }
}

impl From<PortableDocsLifecycleClaimPatternConfig> for DocsLifecycleClaimPatternConfig {
    fn from(config: PortableDocsLifecycleClaimPatternConfig) -> Self {
        Self {
            id: config.id,
            pattern: config.pattern,
            evidence_files: config.evidence_files,
        }
    }
}
