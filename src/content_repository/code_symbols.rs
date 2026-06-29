//! Code-symbol field declarations for repo-native content models.

use super::model::{CodeSymbolSpec, ContentFinding};
use crate::config::config::Config;
use std::collections::BTreeMap;

pub(super) fn code_symbols_by_collection(
    config: &Config,
    findings: &mut Vec<ContentFinding>,
) -> BTreeMap<String, Vec<CodeSymbolSpec>> {
    let mut code_symbols = BTreeMap::<String, Vec<CodeSymbolSpec>>::new();
    for (key, symbol) in &config.code_symbols {
        let Some((source_collection, field)) = key.split_once('.') else {
            findings.push(invalid_code_symbol_key(key));
            continue;
        };
        if source_collection.is_empty() || field.is_empty() || field.contains('.') {
            findings.push(invalid_code_symbol_key(key));
            continue;
        }
        if !config.collections.contains_key(source_collection) {
            findings.push(ContentFinding::new(
                "unknown_content_code_symbol_source",
                None,
                format!("Content code symbol '{key}' references unknown source collection"),
            ));
            continue;
        }
        if symbol
            .provider
            .as_ref()
            .is_some_and(|provider| provider.trim().is_empty())
        {
            findings.push(ContentFinding::new(
                "invalid_content_code_symbol",
                None,
                format!("Content code symbol '{key}' provider must not be empty"),
            ));
            continue;
        }
        code_symbols
            .entry(source_collection.to_string())
            .or_default()
            .push(CodeSymbolSpec {
                field: field.to_string(),
                provider: symbol.provider.clone(),
                many: symbol.many,
            });
    }
    for symbols in code_symbols.values_mut() {
        symbols.sort_by(|left, right| left.field.cmp(&right.field));
    }
    code_symbols
}

fn invalid_code_symbol_key(key: &str) -> ContentFinding {
    ContentFinding::new(
        "invalid_content_code_symbol",
        None,
        format!("Content code symbol key '{key}' must use collection.field syntax"),
    )
}
