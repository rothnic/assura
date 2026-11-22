//! Owner-controlled file budgets, independent of project layout.

use glob::{MatchOptions, Pattern};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerPolicy {
    max_lines: usize,
    #[serde(default)]
    exemptions: Vec<String>,
}

pub(crate) struct OwnerBudget {
    pub(crate) max_lines: usize,
    exemptions: Vec<Pattern>,
}

impl OwnerBudget {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, String> {
        let policy: OwnerPolicy =
            serde_yaml::from_slice(bytes).map_err(|error| error.to_string())?;
        let exemptions = policy
            .exemptions
            .iter()
            .map(|entry| {
                Pattern::new(entry).map_err(|error| format!("invalid exemption {entry:?}: {error}"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            max_lines: policy.max_lines,
            exemptions,
        })
    }

    pub(crate) fn exemption(&self, path: &str) -> Option<&str> {
        self.exemptions
            .iter()
            .find(|pattern| {
                pattern.matches_with(
                    path,
                    MatchOptions {
                        require_literal_separator: true,
                        ..MatchOptions::new()
                    },
                )
            })
            .map(Pattern::as_str)
    }
}
