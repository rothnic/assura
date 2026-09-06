use serde::{Deserialize, Serialize};

/// Experimental policy tier selected by an internal constraint context.
///
/// This tier is not inferred from repository files and does not describe a
/// project's quality or maturity. Supported commands do not expose it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Default,
)]
pub enum MaturityLevel {
    #[default]
    Raw = 0,
    Developing = 1,
    Mature = 2,
    Established = 3,
}

impl std::fmt::Display for MaturityLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw => write!(formatter, "raw"),
            Self::Developing => write!(formatter, "developing"),
            Self::Mature => write!(formatter, "mature"),
            Self::Established => write!(formatter, "established"),
        }
    }
}
