//! Direct-content policy helpers for the LS-Lint fast plan.

use super::rules::EffectiveRules;

/// Whether stripping direct-content constraints would leave effective rules unchanged.
///
/// This mirrors `strip_direct_content_policy` exactly: an explicit empty
/// collection or `false` value remains a direct constraint and must not be
/// treated as a no-op.
pub(super) fn strip_direct_content_policy_is_noop(rules: &EffectiveRules) -> bool {
    rules.limit_children.is_none()
        && rules.files.as_ref().map_or(true, |files| {
            files.required.is_none()
                && files.allowed_names.is_none()
                && files.allowed_patterns.is_none()
                && files.forbidden_patterns.is_none()
                && files.allow_extra.is_none()
                && files.exists.is_none()
        })
        && rules.directories.as_ref().map_or(true, |directories| {
            directories.required.is_none()
                && directories.allowed_names.is_none()
                && directories.allowed_patterns.is_none()
                && directories.forbidden_patterns.is_none()
                && directories.allow_extra.is_none()
                && directories.exists.is_none()
        })
        && rules.self_directory.as_ref().map_or(true, |directory| {
            directory.required.is_none()
                && directory.allowed_names.is_none()
                && directory.allowed_patterns.is_none()
                && directory.forbidden_patterns.is_none()
                && directory.allow_extra.is_none()
                && directory.exists.is_none()
        })
}
