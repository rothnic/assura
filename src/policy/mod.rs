//! Pure repository-policy predicates shared by check execution paths.

pub(crate) mod naming;
#[cfg(feature = "full-cli")]
pub(crate) mod owner_budget;
#[cfg(feature = "yaml-config")]
pub(crate) mod source_selection;
