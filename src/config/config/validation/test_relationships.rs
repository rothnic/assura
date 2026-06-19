//! Source/test relationship semantic validation.

use super::validate_severity;
use super::{validate_identifier, validate_relative_path_text, validate_relative_pattern};
use crate::config::config::{
    TestRelationshipConfig, TestRelationshipFixtureFamilyConfig, TestRelationshipIgnoredTestConfig,
    TestRelationshipSourceConfig,
};
use std::collections::HashSet;
use std::path::Path;

pub(super) fn validate_test_relationship_config(
    policy: &TestRelationshipConfig,
) -> Result<(), String> {
    let context = format!("extensions.test_relationships.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    if policy.relationships.is_empty()
        && policy.fixture_roots.is_empty()
        && policy.ignored_tests.is_empty()
    {
        return Err(format!(
            "{context}: at least one relationship, fixture root, or ignored test is required"
        ));
    }

    validate_relationships(&policy.relationships, &context)?;
    validate_fixture_roots(&policy.fixture_roots, &context)?;
    validate_fixture_families(&policy.fixture_families, &policy.fixture_roots, &context)?;
    validate_ignore_reasons(&policy.allowed_ignore_reasons, &context)?;
    validate_ignored_tests(
        &policy.ignored_tests,
        &policy.allowed_ignore_reasons,
        &context,
    )?;
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn validate_relationships(
    relationships: &[TestRelationshipSourceConfig],
    context: &str,
) -> Result<(), String> {
    let mut sources = HashSet::new();
    for relationship in relationships {
        validate_relative_pattern(
            &relationship.source,
            &format!("{context}.relationships.source"),
        )?;
        if !sources.insert(relationship.source.as_str()) {
            return Err(format!(
                "{context}.relationships.{}: duplicate source relationship",
                relationship.source
            ));
        }
        if relationship.required_tests.is_empty() {
            return Err(format!(
                "{context}.relationships.{}: at least one required test glob is required",
                relationship.source
            ));
        }
        let mut tests = HashSet::new();
        for required_test in &relationship.required_tests {
            validate_relative_pattern(
                required_test,
                &format!("{context}.relationships.required_tests"),
            )?;
            if !tests.insert(required_test.as_str()) {
                return Err(format!(
                    "{context}.relationships.{}.{}: duplicate required test glob",
                    relationship.source, required_test
                ));
            }
        }
    }
    Ok(())
}

fn validate_fixture_roots(roots: &[String], context: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for root in roots {
        validate_relative_path_text(root, &format!("{context}.fixture_roots"))?;
        if !seen.insert(root.as_str()) {
            return Err(format!(
                "{context}.fixture_roots.{root}: duplicate fixture root"
            ));
        }
    }
    Ok(())
}

fn validate_fixture_families(
    families: &[TestRelationshipFixtureFamilyConfig],
    roots: &[String],
    context: &str,
) -> Result<(), String> {
    let mut paths = HashSet::new();
    for family in families {
        validate_relative_path_text(&family.path, &format!("{context}.fixture_families.path"))?;
        if !paths.insert(family.path.as_str()) {
            return Err(format!(
                "{context}.fixture_families.{}: duplicate fixture family",
                family.path
            ));
        }
        validate_text(&family.owner, &format!("{context}.fixture_families.owner"))?;
        validate_text(
            &family.purpose,
            &format!("{context}.fixture_families.purpose"),
        )?;
        if !roots.is_empty()
            && !roots
                .iter()
                .any(|root| Path::new(&family.path).starts_with(Path::new(root)))
        {
            return Err(format!(
                "{context}.fixture_families.{}: fixture family must be under one of fixture_roots",
                family.path
            ));
        }
    }
    Ok(())
}

fn validate_ignore_reasons(reasons: &[String], context: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for reason in reasons {
        validate_identifier(reason, &format!("{context}.allowed_ignore_reasons"))?;
        if !seen.insert(reason.as_str()) {
            return Err(format!(
                "{context}.allowed_ignore_reasons.{reason}: duplicate ignored-test reason"
            ));
        }
    }
    Ok(())
}

fn validate_ignored_tests(
    tests: &[TestRelationshipIgnoredTestConfig],
    allowed_reasons: &[String],
    context: &str,
) -> Result<(), String> {
    let allowed = allowed_reasons
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut accepted_tests = HashSet::new();
    for test in tests {
        validate_relative_pattern(&test.path, &format!("{context}.ignored_tests.path"))?;
        validate_text(&test.test, &format!("{context}.ignored_tests.test"))?;
        let key = format!("{}::{}", test.path, test.test);
        if !accepted_tests.insert(key) {
            return Err(format!(
                "{context}.ignored_tests.{}::{}: duplicate ignored test",
                test.path, test.test
            ));
        }
        validate_identifier(&test.reason, &format!("{context}.ignored_tests.reason"))?;
        if !allowed.contains(test.reason.as_str()) {
            return Err(format!(
                "{context}.ignored_tests.{}: reason `{}` is not listed in allowed_ignore_reasons",
                test.path, test.reason
            ));
        }
    }
    Ok(())
}

fn validate_text(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{context}: value must not be empty"));
    }
    Ok(())
}
