//! Pure repository-policy predicates shared by check execution paths.

pub(crate) mod naming;
#[allow(
    dead_code,
    reason = "The policy pipeline will consume this selector in a later slice"
)]
pub(crate) mod source_selection;
