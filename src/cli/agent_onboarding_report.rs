//! Serializable report types for `assura agent onboard`.

use super::agent_lifecycle::{LifecycleProfile, RankedNextAction};
use super::agent_onboarding::{DetectedSection, OnboardingReview};
use super::OutputFormat;
use serde::Serialize;
use std::io::{self, Write};

pub(super) fn write_report(
    report: &RenderedOnboardingReport,
    writer: &mut impl Write,
) -> io::Result<()> {
    match report.format {
        OutputFormat::Json => write_serialized_report(writer, &report.report)?,
        OutputFormat::Yaml => {
            serde_yaml::to_writer(&mut *writer, &report.report).map_err(io::Error::other)?
        }
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => {
            writer.write_all(report.render_text().as_bytes())?
        }
    }
    writer.write_all(b"\n")
}

fn write_serialized_report(writer: &mut impl Write, report: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, report).map_err(io::Error::other)
}

#[derive(Serialize)]
pub(super) struct RenderedOnboardingReport {
    #[serde(flatten)]
    pub(super) report: OnboardingReport,
    #[serde(skip)]
    pub(super) format: OutputFormat,
}

impl RenderedOnboardingReport {
    fn render_text(&self) -> String {
        let report = &self.report;
        [
            "Assura agent onboarding".to_string(),
            onboarding_row(
                "Version",
                format!("assura {}", report.installed.assura_version),
            ),
            onboarding_row(
                "Project",
                format!(
                    "{} confidence={}",
                    report.detected.project_type, report.detected.project_confidence
                ),
            ),
            onboarding_row(
                "Agent",
                format!(
                    "{} confidence={}",
                    report.detected.agent_harness, report.detected.agent_confidence
                ),
            ),
            onboarding_row(
                "Policy",
                report
                    .rule_recommendations
                    .iter()
                    .map(|item| format!("{} -> {} ({})", item.preset, item.local_rule, item.status))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            onboarding_row(
                "Host",
                format!(
                    "{} generated={} activated={} verified={} conflicted={}",
                    report.integration.agent,
                    report.integration.generated,
                    report.integration.activated,
                    report.integration.verified,
                    report.integration.conflicted
                ),
            ),
            onboarding_row(
                "Content",
                format!("{}={}", report.content.template, report.content.status),
            ),
            onboarding_row(
                "Lifecycle",
                report
                    .lifecycle_profiles
                    .iter()
                    .map(|item| format!("{}={}", item.name, item.mode))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            onboarding_row(
                "Verified",
                report
                    .verified
                    .iter()
                    .map(|item| format!("{}={}", item.name, item.status))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            onboarding_row(
                "Review",
                format!(
                    "{} blocking={} advisory={} inactive_signals={}",
                    report.review.status,
                    report.review.blocking,
                    report.review.advisory,
                    report.review.inactive
                ),
            ),
            onboarding_row(
                "Deferred",
                report
                    .inactive
                    .iter()
                    .map(|item| item.name)
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            onboarding_row(
                "Next",
                report
                    .next_actions
                    .first()
                    .map(|action| action.action)
                    .unwrap_or("read agent-next.md")
                    .to_string(),
            ),
            onboarding_row("Packet", ".assura/onboarding/agent-next.md".to_string()),
        ]
        .join("\n")
    }
}

fn onboarding_row(label: &str, value: String) -> String {
    format!("{label:<12} {value}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "injected output failure",
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn serialized_report_returns_the_writer_error() {
        let mut writer = FailingWriter;
        let error = write_serialized_report(&mut writer, &serde_json::json!({"schema": "test"}))
            .expect_err("an output failure must not become an empty success payload");

        assert!(
            error.to_string().contains("injected output failure"),
            "writer failure was not preserved: {error}"
        );
    }
}

#[derive(Serialize)]
pub(super) struct OnboardingReport {
    pub(super) schema: &'static str,
    pub(super) project_root: String,
    pub(super) installed: InstalledSection,
    pub(super) detected: DetectedSection,
    pub(super) rule_recommendations: Vec<RuleRecommendation>,
    pub(super) integration: IntegrationSection,
    pub(super) content: ContentSection,
    pub(super) lifecycle_profiles: Vec<LifecycleProfile>,
    pub(super) files: Vec<FileAction>,
    pub(super) verified: Vec<CheckItem>,
    pub(super) review: OnboardingReview,
    pub(super) inactive: Vec<CheckItem>,
    pub(super) next_actions: Vec<RankedNextAction>,
}

#[derive(Serialize)]
pub(super) struct RuleRecommendation {
    pub(super) preset: &'static str,
    pub(super) local_rule: &'static str,
    pub(super) status: &'static str,
    pub(super) reason: String,
    pub(super) includes: Vec<&'static str>,
}

#[derive(Serialize)]
pub(super) struct InstalledSection {
    pub(super) assura_version: &'static str,
    pub(super) config: &'static str,
    pub(super) onboarding_packet: &'static str,
}

#[derive(Serialize)]
pub(super) struct IntegrationSection {
    pub(super) status: &'static str,
    pub(super) agent: &'static str,
    pub(super) mode: &'static str,
    pub(super) generated: bool,
    pub(super) activated: bool,
    pub(super) verified: bool,
    pub(super) conflicted: bool,
    pub(super) detail: &'static str,
}

#[derive(Serialize)]
pub(super) struct ContentSection {
    pub(super) template: &'static str,
    pub(super) status: &'static str,
    pub(super) detail: &'static str,
}

#[derive(Serialize)]
pub(super) struct FileAction {
    pub(super) path: &'static str,
    pub(super) action: &'static str,
    pub(super) existed: bool,
    pub(super) required: bool,
}

#[derive(Clone, Serialize)]
pub(super) struct CheckItem {
    pub(super) name: &'static str,
    pub(super) status: &'static str,
    pub(super) detail: &'static str,
}
