//! Serializable report types for `assura agent onboard`.

use super::agent_lifecycle::{LifecycleProfile, RankedNextAction};
use super::agent_onboarding::{DetectedSection, OnboardingReview};
use super::OutputFormat;
use serde::Serialize;

pub(super) fn render_report(report: &RenderedOnboardingReport) -> String {
    match report.format {
        OutputFormat::Json => serde_json::to_string_pretty(&report.report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(&report.report).unwrap_or_default(),
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => report.render_text(),
    }
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
        format!(
            "Assura agent onboarding\ninstalled: assura {}\ndetected: project={} agent={} confidence={}\nrules: {}\ncontent: {}={}\nlifecycle: {}\nverified: {}\nreview: {} blocking={} advisory={} inactive={}\ninactive: {}\nnext: {}\npacket: .assura/onboarding/agent-next.md",
            report.installed.assura_version,
            report.detected.project_type,
            report.detected.agent_harness,
            report.detected.agent_confidence,
            report
                .rule_recommendations
                .iter()
                .map(|item| format!("{}->{} ({})", item.preset, item.local_rule, item.status))
                .collect::<Vec<_>>()
                .join(", "),
            report.content.template,
            report.content.status,
            report
                .lifecycle_profiles
                .iter()
                .map(|item| format!("{}={}", item.name, item.mode))
                .collect::<Vec<_>>()
                .join(", "),
            report
                .verified
                .iter()
                .map(|item| format!("{}={}", item.name, item.status))
                .collect::<Vec<_>>()
                .join(", "),
            report.review.status,
            report.review.blocking,
            report.review.advisory,
            report.review.inactive,
            report
                .inactive
                .iter()
                .map(|item| item.name)
                .collect::<Vec<_>>()
                .join(", "),
            report
                .next_actions
                .first()
                .map(|action| action.action)
                .unwrap_or("read agent-next.md")
        )
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
