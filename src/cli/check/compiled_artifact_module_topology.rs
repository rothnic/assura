/// Portable module-topology extension config for compiled artifacts.
use crate::config::config::ModuleTopologyModuleConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableModuleTopologyConfig {
    id: String,
    modules: Vec<PortableModuleTopologyModuleConfig>,
    rust_exports: Vec<String>,
    severity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableModuleTopologyModuleConfig {
    family: String,
    status: String,
    owner: String,
    purpose: String,
    roots: Vec<String>,
    public_exports: Vec<String>,
    visibility: Option<String>,
}

impl From<ModuleTopologyConfig> for PortableModuleTopologyConfig {
    fn from(config: ModuleTopologyConfig) -> Self {
        Self {
            id: config.id,
            modules: config.modules.into_iter().map(Into::into).collect(),
            rust_exports: config.rust_exports,
            severity: config.severity,
        }
    }
}

impl From<PortableModuleTopologyConfig> for ModuleTopologyConfig {
    fn from(config: PortableModuleTopologyConfig) -> Self {
        Self {
            id: config.id,
            modules: config.modules.into_iter().map(Into::into).collect(),
            rust_exports: config.rust_exports,
            severity: config.severity,
        }
    }
}

impl From<ModuleTopologyModuleConfig> for PortableModuleTopologyModuleConfig {
    fn from(config: ModuleTopologyModuleConfig) -> Self {
        Self {
            family: config.family,
            status: config.status,
            owner: config.owner,
            purpose: config.purpose,
            roots: config.roots,
            public_exports: config.public_exports,
            visibility: config.visibility,
        }
    }
}

impl From<PortableModuleTopologyModuleConfig> for ModuleTopologyModuleConfig {
    fn from(config: PortableModuleTopologyModuleConfig) -> Self {
        Self {
            family: config.family,
            status: config.status,
            owner: config.owner,
            purpose: config.purpose,
            roots: config.roots,
            public_exports: config.public_exports,
            visibility: config.visibility,
        }
    }
}
