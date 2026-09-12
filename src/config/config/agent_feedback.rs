//! Opt-in compact feedback delivery configuration.

use serde::{Deserialize, Serialize};

/// Automatic compact feedback mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentFeedbackMode {
    /// Do not collect or inject trajectory feedback.
    Off,
    /// Deliver feedback when a configured signal crosses a threshold.
    #[default]
    Threshold,
    /// Deliver one fresh line at a configured cadence.
    Periodic,
}

impl AgentFeedbackMode {
    /// Stable machine-readable mode label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Threshold => "threshold",
            Self::Periodic => "periodic",
        }
    }
}

/// Project-local compact automatic feedback policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackConfig {
    /// Automatic delivery mode.
    #[serde(default)]
    pub mode: AgentFeedbackMode,
    /// Maximum UTF-8 bytes in one routine line.
    #[serde(default = "default_max_bytes")]
    pub max_bytes: u16,
    /// Minimum spacing between routine messages.
    #[serde(default = "default_min_interval")]
    pub min_interval_seconds: u64,
    /// Maximum routine messages in the rolling hour.
    #[serde(default = "default_max_messages")]
    pub max_messages_per_hour: u16,
    /// Maximum routine bytes in the rolling hour.
    #[serde(default = "default_max_hourly_bytes")]
    pub max_bytes_per_hour: u32,
    /// Delay before a bounded reminder becomes eligible.
    #[serde(default = "default_reminder")]
    pub reminder_seconds: u64,
    /// Maximum reminders for one persistent episode.
    #[serde(default = "default_max_reminders")]
    pub max_reminders_per_episode: u8,
    /// Periodic delivery cadence.
    #[serde(default = "default_periodic")]
    pub periodic_seconds: u64,
    /// Bounded collection settings.
    #[serde(default)]
    pub collection: AgentFeedbackCollectionConfig,
    /// Git trajectory classification and signal settings.
    #[serde(default)]
    pub trajectory: AgentFeedbackTrajectoryConfig,
}

impl Default for AgentFeedbackConfig {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            max_bytes: default_max_bytes(),
            min_interval_seconds: default_min_interval(),
            max_messages_per_hour: default_max_messages(),
            max_bytes_per_hour: default_max_hourly_bytes(),
            reminder_seconds: default_reminder(),
            max_reminders_per_episode: default_max_reminders(),
            periodic_seconds: default_periodic(),
            collection: Default::default(),
            trajectory: Default::default(),
        }
    }
}

/// Collector bounds and freshness policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackCollectionConfig {
    /// Event coalescing delay.
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
    /// Minimum time between refreshes.
    #[serde(default = "default_refresh")]
    pub min_refresh_seconds: u64,
    /// Maximum age of an automatic snapshot.
    #[serde(default = "default_stale_after")]
    pub stale_after_seconds: u64,
    /// Collector wall-clock timeout.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Maximum commits examined by one collector.
    #[serde(default = "default_max_commits")]
    pub max_commits: u16,
}
impl Default for AgentFeedbackCollectionConfig {
    fn default() -> Self {
        Self {
            debounce_ms: default_debounce(),
            min_refresh_seconds: default_refresh(),
            stale_after_seconds: default_stale_after(),
            timeout_ms: default_timeout(),
            max_commits: default_max_commits(),
        }
    }
}

/// Trajectory category and signal configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackTrajectoryConfig {
    /// Integration ref.
    #[serde(default = "default_integration_ref")]
    pub integration_ref: String,
    /// Exactly one bounded trajectory window.
    #[serde(default)]
    pub window: AgentFeedbackWindowConfig,
    /// Selected compact metrics.
    #[serde(default = "default_metrics")]
    pub metrics: Vec<String>,
    /// Source path globs.
    #[serde(default = "default_source_paths")]
    pub source_paths: Vec<String>,
    /// Test path globs.
    #[serde(default = "default_test_paths")]
    pub test_paths: Vec<String>,
    /// Coordination path globs.
    #[serde(default = "default_coordination_paths")]
    pub coordination_paths: Vec<String>,
    /// Generated path globs.
    #[serde(default = "default_generated_paths")]
    pub generated_paths: Vec<String>,
    /// Signal thresholds.
    #[serde(default)]
    pub signals: AgentFeedbackSignalsConfig,
}
impl Default for AgentFeedbackTrajectoryConfig {
    fn default() -> Self {
        Self {
            integration_ref: default_integration_ref(),
            window: Default::default(),
            metrics: default_metrics(),
            source_paths: default_source_paths(),
            test_paths: default_test_paths(),
            coordination_paths: default_coordination_paths(),
            generated_paths: default_generated_paths(),
            signals: Default::default(),
        }
    }
}

/// Commit- or time-bounded trajectory window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackWindowConfig {
    /// Integrated committer-time window.
    #[serde(default)]
    pub minutes: Option<u64>,
    /// Integrated first-parent commit window.
    #[serde(default)]
    pub commits: Option<u64>,
}
impl Default for AgentFeedbackWindowConfig {
    fn default() -> Self {
        Self {
            minutes: Some(30),
            commits: None,
        }
    }
}

/// Configured signal families.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackSignalsConfig {
    /// Unintegrated-work signal.
    #[serde(default)]
    pub unintegrated: AgentFeedbackUnintegratedSignal,
    /// Coordination-only commit signal.
    #[serde(default)]
    pub coordination: AgentFeedbackCoordinationSignal,
    /// Pending patch-size signal.
    #[serde(default)]
    pub patch_size: AgentFeedbackPatchSignal,
}

/// Pending-age signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackUnintegratedSignal {
    /// Enable the signal.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Entry age in minutes.
    #[serde(default = "default_pending_minutes")]
    pub pending_minutes: u64,
    /// Material step in minutes.
    #[serde(default = "default_pending_step")]
    pub step_minutes: u64,
    /// Clean duration needed to clear.
    #[serde(default = "default_clear_seconds")]
    pub clear_after_clean_seconds: u64,
}
impl Default for AgentFeedbackUnintegratedSignal {
    fn default() -> Self {
        Self {
            enabled: true,
            pending_minutes: 30,
            step_minutes: 30,
            clear_after_clean_seconds: 60,
        }
    }
}

/// Coordination-only signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackCoordinationSignal {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_coordination_commits")]
    pub commits: u64,
    #[serde(default = "default_coordination_entry")]
    pub min_only_coordination_commits: u64,
    #[serde(default = "default_coordination_clear")]
    pub clear_below_only_coordination_commits: u64,
}
impl Default for AgentFeedbackCoordinationSignal {
    fn default() -> Self {
        Self {
            enabled: false,
            commits: 20,
            min_only_coordination_commits: 19,
            clear_below_only_coordination_commits: 16,
        }
    }
}

/// Patch-size signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AgentFeedbackPatchSignal {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_patch_lines")]
    pub changed_lines: u64,
    #[serde(default = "default_patch_step")]
    pub step_lines: u64,
    #[serde(default = "default_patch_clear")]
    pub clear_below_changed_lines: u64,
}
impl Default for AgentFeedbackPatchSignal {
    fn default() -> Self {
        Self {
            enabled: false,
            changed_lines: 1000,
            step_lines: 500,
            clear_below_changed_lines: 800,
        }
    }
}

const fn default_true() -> bool {
    true
}
const fn default_max_bytes() -> u16 {
    256
}
const fn default_min_interval() -> u64 {
    600
}
const fn default_max_messages() -> u16 {
    4
}
const fn default_max_hourly_bytes() -> u32 {
    1024
}
const fn default_reminder() -> u64 {
    1800
}
const fn default_max_reminders() -> u8 {
    1
}
const fn default_periodic() -> u64 {
    900
}
const fn default_debounce() -> u64 {
    1000
}
const fn default_refresh() -> u64 {
    30
}
const fn default_stale_after() -> u64 {
    120
}
const fn default_timeout() -> u64 {
    2000
}
const fn default_max_commits() -> u16 {
    500
}
fn default_integration_ref() -> String {
    "origin/master".into()
}
fn default_metrics() -> Vec<String> {
    [
        "integrated_commits",
        "source_lines",
        "pending_age",
        "pending_commits",
        "pending_lines",
        "dirty_files",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
fn default_source_paths() -> Vec<String> {
    ["src/**", "xtask/src/**", "website/src/**"]
        .into_iter()
        .map(String::from)
        .collect()
}
fn default_test_paths() -> Vec<String> {
    vec!["tests/**".into()]
}
fn default_coordination_paths() -> Vec<String> {
    [".trellis/**", ".agents/**", ".codex/**", "docs/goals/**"]
        .into_iter()
        .map(String::from)
        .collect()
}
fn default_generated_paths() -> Vec<String> {
    ["target/**", "dist/**"]
        .into_iter()
        .map(String::from)
        .collect()
}
const fn default_pending_minutes() -> u64 {
    30
}
const fn default_pending_step() -> u64 {
    30
}
const fn default_clear_seconds() -> u64 {
    60
}
const fn default_coordination_commits() -> u64 {
    20
}
const fn default_coordination_entry() -> u64 {
    19
}
const fn default_coordination_clear() -> u64 {
    16
}
const fn default_patch_lines() -> u64 {
    1000
}
const fn default_patch_step() -> u64 {
    500
}
const fn default_patch_clear() -> u64 {
    800
}
