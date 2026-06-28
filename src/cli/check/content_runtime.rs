//! Adapts repo-native content runtime findings into structure-check reports.

use super::{StructureCheckReport, StructureChecker, StructureViolation};
use crate::content_repository::{ContentFinding, ContentRepository};
use std::path::{Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_content_runtime(&self, report: &mut StructureCheckReport) {
        if !self.has_content_runtime_config() {
            return;
        }

        match ContentRepository::from_config(&self.project_root, &self.config) {
            Ok(repository) => {
                for finding in repository.validate(&self.project_root).findings {
                    self.push_content_finding(report, finding);
                }
            }
            Err(findings) => {
                for finding in findings {
                    self.push_content_finding(report, finding);
                }
            }
        }
    }

    pub(super) fn has_content_runtime_config(&self) -> bool {
        self.config.models.is_some()
            || !self.config.collections.is_empty()
            || !self.config.relations.is_empty()
    }

    fn push_content_finding(&self, report: &mut StructureCheckReport, finding: ContentFinding) {
        let path = content_finding_path(&finding, &report.config_path, &self.project_root);

        report.violations.push(StructureViolation::new(
            path,
            format!("content_runtime:{}", finding.code),
            content_finding_message(&finding),
            "high",
        ));
    }
}

fn content_finding_path(
    finding: &ContentFinding,
    config_path: &Path,
    project_root: &Path,
) -> PathBuf {
    finding.path.clone().unwrap_or_else(|| {
        config_path
            .strip_prefix(project_root)
            .unwrap_or(config_path)
            .to_path_buf()
    })
}

fn content_finding_message(finding: &ContentFinding) -> String {
    let mut context = Vec::new();
    if let Some(object_type) = finding.object_type.as_deref() {
        context.push(format!("object_type={object_type}"));
    }
    if let Some(field) = finding.field.as_deref() {
        context.push(format!("field={field}"));
    }
    if let Some(referenced_object) = finding.referenced_object.as_deref() {
        context.push(format!("referenced_object={referenced_object}"));
    }

    if context.is_empty() {
        finding.message.clone()
    } else {
        format!("{} ({})", finding.message, context.join(", "))
    }
}
