use serde::{Deserialize, Serialize};

/// A controlled project-local script-backed computed check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComputedCheckConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Project-local script path, relative to the project root.
    pub script: String,
    /// Optional Windows-specific project-local script path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_script: Option<String>,
    /// Literal arguments passed to the script without shell expansion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Maximum runtime in milliseconds.
    #[serde(
        default = "default_timeout_ms",
        skip_serializing_if = "is_default_timeout"
    )]
    pub timeout_ms: u64,
    /// Optional default diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

fn default_timeout_ms() -> u64 {
    5_000
}

fn is_default_timeout(value: &u64) -> bool {
    *value == default_timeout_ms()
}
