//! YAML notation routing and normalization for configuration loading.

use super::Config;
use crate::cli::config::{ConfigError, ConfigResult};
use crate::config::config::{normalize_structure_config_value, validate_config_semantics};

pub(super) fn parse_yaml_value(content: &str) -> ConfigResult<serde_yaml::Value> {
    serde_yaml::from_str(content).map_err(|error| ConfigError::Yaml(error.to_string()))
}

pub(super) fn parse_normalized(content: &str) -> ConfigResult<Config> {
    parse_normalized_value(parse_yaml_value(content)?)
}

pub(super) fn parse_normalized_value(value: serde_yaml::Value) -> ConfigResult<Config> {
    let value = normalize_structure_config_value(value).map_err(ConfigError::Invalid)?;
    let config: Config =
        serde_yaml::from_value(value).map_err(|error| ConfigError::Yaml(error.to_string()))?;
    validate_config_semantics(&config).map_err(ConfigError::Invalid)?;
    Ok(config)
}

pub(super) fn has_top_level_rules(value: &serde_yaml::Value) -> bool {
    value
        .as_mapping()
        .is_some_and(|mapping| mapping.contains_key(serde_yaml::Value::String("rules".to_string())))
}

#[cfg(test)]
mod tests {
    use crate::config::loader::ConfigLoader;

    #[test]
    fn top_level_rules_key_variants_always_use_notation_normalization() {
        for (index, yaml) in [
            "rules:\n  \"@source-file\":\n    naming: kebab-case\nstructure: {}\n",
            "rules :\n  \"@source-file\":\n    naming: kebab-case\nstructure: {}\n",
            "\"rules\" :\n  \"@source-file\":\n    naming: kebab-case\nstructure: {}\n",
            "'rules' :\n  \"@source-file\":\n    naming: kebab-case\nstructure: {}\n",
            " rules :\n   \"@source-file\":\n     naming: kebab-case\n structure: {}\n",
        ]
        .into_iter()
        .enumerate()
        {
            let error = ConfigLoader::parse(yaml)
                .err()
                .unwrap_or_else(|| panic!("top-level rules case {index} unexpectedly parsed"))
                .to_string();
            assert!(
                error.contains("replace '@source-file' with 'source-file'"),
                "top-level rules case {index} returned an unexpected error: {error}"
            );
        }
    }
}
