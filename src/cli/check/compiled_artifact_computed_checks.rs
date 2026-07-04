use crate::config::config::ComputedCheckConfig;

/// Binary-safe computed-check policy stored in compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableComputedCheckConfig {
    id: String,
    script: String,
    windows_script: Option<String>,
    args: Vec<String>,
    timeout_ms: u64,
    severity: Option<String>,
}

impl From<ComputedCheckConfig> for PortableComputedCheckConfig {
    fn from(config: ComputedCheckConfig) -> Self {
        Self {
            id: config.id,
            script: config.script,
            windows_script: config.windows_script,
            args: config.args,
            timeout_ms: config.timeout_ms,
            severity: config.severity,
        }
    }
}

impl From<PortableComputedCheckConfig> for ComputedCheckConfig {
    fn from(config: PortableComputedCheckConfig) -> Self {
        Self {
            id: config.id,
            script: config.script,
            windows_script: config.windows_script,
            args: config.args,
            timeout_ms: config.timeout_ms,
            severity: config.severity,
        }
    }
}
