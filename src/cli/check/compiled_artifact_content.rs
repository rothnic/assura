/// Portable content-runtime config stored inside compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableContentModelConfig {
    source: Option<String>,
    validation_artifact: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableContentCollectionConfig {
    class_name: String,
    path: String,
    adapter: String,
    data: Option<String>,
    body: Option<String>,
    id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableContentCodeSymbolConfig {
    provider: Option<String>,
    many: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableContentRelationConfig {
    target: Option<String>,
    targets: Vec<String>,
    many: bool,
    required: bool,
    acyclic: bool,
}

impl From<ContentModelConfig> for PortableContentModelConfig {
    fn from(config: ContentModelConfig) -> Self {
        Self {
            source: config.source,
            validation_artifact: config.validation_artifact,
        }
    }
}

impl From<PortableContentModelConfig> for ContentModelConfig {
    fn from(config: PortableContentModelConfig) -> Self {
        Self {
            source: config.source,
            validation_artifact: config.validation_artifact,
        }
    }
}

impl From<ContentCollectionConfig> for PortableContentCollectionConfig {
    fn from(config: ContentCollectionConfig) -> Self {
        Self {
            class_name: config.class_name,
            path: config.path,
            adapter: config.adapter,
            data: config.data,
            body: config.body,
            id: config.id,
        }
    }
}

impl From<PortableContentCollectionConfig> for ContentCollectionConfig {
    fn from(config: PortableContentCollectionConfig) -> Self {
        Self {
            class_name: config.class_name,
            path: config.path,
            adapter: config.adapter,
            data: config.data,
            body: config.body,
            id: config.id,
        }
    }
}

impl From<ContentCodeSymbolConfig> for PortableContentCodeSymbolConfig {
    fn from(config: ContentCodeSymbolConfig) -> Self {
        Self {
            provider: config.provider,
            many: config.many,
        }
    }
}

impl From<PortableContentCodeSymbolConfig> for ContentCodeSymbolConfig {
    fn from(config: PortableContentCodeSymbolConfig) -> Self {
        Self {
            provider: config.provider,
            many: config.many,
        }
    }
}

impl From<ContentRelationConfig> for PortableContentRelationConfig {
    fn from(config: ContentRelationConfig) -> Self {
        Self {
            target: config.target,
            targets: config.targets,
            many: config.many,
            required: config.required,
            acyclic: config.acyclic,
        }
    }
}

impl From<PortableContentRelationConfig> for ContentRelationConfig {
    fn from(config: PortableContentRelationConfig) -> Self {
        Self {
            target: config.target,
            targets: config.targets,
            many: config.many,
            required: config.required,
            acyclic: config.acyclic,
        }
    }
}
