use serde::{Deserialize, Serialize};

use super::signal::MaturitySignal;

/// Maturity levels for projects
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Default,
)]
pub enum MaturityLevel {
    /// Project is in early stages, minimal structure
    #[default]
    Raw = 0,
    /// Project has basic structure but may lack polish
    Developing = 1,
    /// Project is well-structured and maintained
    Mature = 2,
    /// Project is highly mature with comprehensive tooling
    Established = 3,
}

impl MaturityLevel {
    /// Get the threshold score for this level (0.0 - 1.0)
    pub fn threshold(self) -> f64 {
        match self {
            MaturityLevel::Raw => 0.0,
            MaturityLevel::Developing => 0.3,
            MaturityLevel::Mature => 0.6,
            MaturityLevel::Established => 0.85,
        }
    }

    /// Get a human-readable description
    pub fn description(self) -> &'static str {
        match self {
            MaturityLevel::Raw => {
                "Project is in early development stages with minimal structure and tooling"
            }
            MaturityLevel::Developing => {
                "Project has basic structure and some tooling but needs more polish"
            }
            MaturityLevel::Mature => {
                "Project is well-structured with good tooling and maintenance practices"
            }
            MaturityLevel::Established => {
                "Project is highly mature with comprehensive tooling and established practices"
            }
        }
    }

    /// Get the next higher maturity level if any
    pub fn next(self) -> Option<Self> {
        match self {
            MaturityLevel::Raw => Some(MaturityLevel::Developing),
            MaturityLevel::Developing => Some(MaturityLevel::Mature),
            MaturityLevel::Mature => Some(MaturityLevel::Established),
            MaturityLevel::Established => None,
        }
    }
}

impl std::fmt::Display for MaturityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaturityLevel::Raw => write!(f, "raw"),
            MaturityLevel::Developing => write!(f, "developing"),
            MaturityLevel::Mature => write!(f, "mature"),
            MaturityLevel::Established => write!(f, "established"),
        }
    }
}

/// A comprehensive report of project maturity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaturityReport {
    /// The assigned maturity level
    pub level: MaturityLevel,
    /// Overall maturity score (0.0 - 1.0)
    pub score: f64,
    /// Confidence in this assessment (0.0 - 1.0)
    pub confidence: f64,
    /// All signals that were collected
    pub signals: Vec<MaturitySignal>,
    /// Detailed breakdown by category
    pub category_scores: CategoryScores,
    /// Recommendations for improvement
    pub recommendations: Vec<Recommendation>,
    /// Timestamp of the assessment
    pub assessed_at: u64,
}

impl MaturityReport {
    pub fn new(
        level: MaturityLevel,
        score: f64,
        confidence: f64,
        signals: Vec<MaturitySignal>,
    ) -> Self {
        Self {
            level,
            score,
            confidence,
            signals,
            category_scores: CategoryScores::default(),
            recommendations: Vec::new(),
            assessed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_category_scores(mut self, scores: CategoryScores) -> Self {
        self.category_scores = scores;
        self
    }

    pub fn with_recommendations(mut self, recommendations: Vec<Recommendation>) -> Self {
        self.recommendations = recommendations;
        self
    }
}

/// Scores broken down by signal category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryScores {
    pub git: f64,
    pub filesystem: f64,
    pub environment: f64,
}

impl CategoryScores {
    pub fn average(&self) -> f64 {
        (self.git + self.filesystem + self.environment) / 3.0
    }

    pub fn min(&self) -> f64 {
        self.git.min(self.filesystem).min(self.environment)
    }
}

/// A recommendation for improving maturity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String,
    pub priority: Priority,
    pub message: String,
    pub actionable: bool,
}

impl Recommendation {
    pub fn new(
        category: impl Into<String>,
        priority: Priority,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            priority,
            message: message.into(),
            actionable: true,
        }
    }

    pub fn not_actionable(mut self) -> Self {
        self.actionable = false;
        self
    }
}

/// Priority levels for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
            Priority::Critical => write!(f, "critical"),
        }
    }
}

/// The decision engine that evaluates signals and determines maturity
#[derive(Debug, Clone)]
pub struct MaturityDecisionEngine {
    /// Minimum confidence required for a level
    min_confidence_threshold: f64,
    /// Whether to require minimum scores in all categories
    require_balanced_scores: bool,
    /// Weight for git signals
    git_weight: f64,
    /// Weight for filesystem signals
    filesystem_weight: f64,
    /// Weight for environment signals
    environment_weight: f64,
}

impl MaturityDecisionEngine {
    pub fn new() -> Self {
        Self {
            min_confidence_threshold: 0.5,
            require_balanced_scores: true,
            git_weight: 1.0,
            filesystem_weight: 1.0,
            environment_weight: 1.0,
        }
    }

    pub fn with_min_confidence(mut self, threshold: f64) -> Self {
        self.min_confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_balanced_requirement(mut self, require: bool) -> Self {
        self.require_balanced_scores = require;
        self
    }

    pub fn with_weights(mut self, git: f64, filesystem: f64, environment: f64) -> Self {
        self.git_weight = git.max(0.0);
        self.filesystem_weight = filesystem.max(0.0);
        self.environment_weight = environment.max(0.0);
        self
    }

    /// Evaluate signals and produce a maturity report
    pub fn evaluate(&self, signals: &[MaturitySignal]) -> MaturityReport {
        if signals.is_empty() {
            return MaturityReport::new(MaturityLevel::Raw, 0.0, 0.0, Vec::new());
        }

        // Calculate category scores
        let category_scores = self.calculate_category_scores(signals);

        // Calculate overall score
        let overall_score = self.calculate_overall_score(&category_scores, signals);

        // Determine maturity level
        let level = self.determine_level(overall_score, &category_scores);

        // Calculate confidence
        let confidence = self.calculate_confidence(signals);

        // Generate recommendations
        let recommendations = self.generate_recommendations(signals, &category_scores, level);

        MaturityReport::new(level, overall_score, confidence, signals.to_vec())
            .with_category_scores(category_scores)
            .with_recommendations(recommendations)
    }

    fn calculate_category_scores(&self, signals: &[MaturitySignal]) -> CategoryScores {
        use super::signal::SignalType;

        let mut git_sum = 0.0;
        let mut git_count = 0;
        let mut fs_sum = 0.0;
        let mut fs_count = 0;
        let mut env_sum = 0.0;
        let mut env_count = 0;

        for signal in signals {
            let weighted = signal.weighted_value();
            match signal.signal_type {
                SignalType::Git => {
                    git_sum += weighted;
                    git_count += 1;
                }
                SignalType::Filesystem => {
                    fs_sum += weighted;
                    fs_count += 1;
                }
                SignalType::Environment => {
                    env_sum += weighted;
                    env_count += 1;
                }
            }
        }

        CategoryScores {
            git: if git_count > 0 {
                git_sum / git_count as f64
            } else {
                0.0
            },
            filesystem: if fs_count > 0 {
                fs_sum / fs_count as f64
            } else {
                0.0
            },
            environment: if env_count > 0 {
                env_sum / env_count as f64
            } else {
                0.0
            },
        }
    }

    fn calculate_overall_score(
        &self,
        category_scores: &CategoryScores,
        signals: &[MaturitySignal],
    ) -> f64 {
        // Weighted average of categories
        let total_weight = self.git_weight + self.filesystem_weight + self.environment_weight;

        let weighted_score = (category_scores.git * self.git_weight
            + category_scores.filesystem * self.filesystem_weight
            + category_scores.environment * self.environment_weight)
            / total_weight;

        // Adjust based on signal confidence
        let avg_confidence = if signals.is_empty() {
            0.0
        } else {
            signals.iter().map(|s| s.confidence).sum::<f64>() / signals.len() as f64
        };

        weighted_score * avg_confidence
    }

    fn determine_level(&self, score: f64, category_scores: &CategoryScores) -> MaturityLevel {
        // Check for balanced scores requirement
        if self.require_balanced_scores {
            let min_score = category_scores.min();
            let max_score = category_scores
                .git
                .max(category_scores.filesystem)
                .max(category_scores.environment);

            // If there's a large imbalance, cap the level
            if max_score - min_score > 0.5 {
                // Find the highest level possible with balanced improvement
                return match score {
                    s if s >= MaturityLevel::Established.threshold() => MaturityLevel::Mature,
                    s if s >= MaturityLevel::Mature.threshold() => MaturityLevel::Developing,
                    s if s >= MaturityLevel::Developing.threshold() => MaturityLevel::Developing,
                    _ => MaturityLevel::Raw,
                };
            }
        }

        // Standard threshold-based assignment
        match score {
            s if s >= MaturityLevel::Established.threshold() => MaturityLevel::Established,
            s if s >= MaturityLevel::Mature.threshold() => MaturityLevel::Mature,
            s if s >= MaturityLevel::Developing.threshold() => MaturityLevel::Developing,
            _ => MaturityLevel::Raw,
        }
    }

    fn calculate_confidence(&self, signals: &[MaturitySignal]) -> f64 {
        if signals.is_empty() {
            return 0.0;
        }

        // Base confidence on signal coverage
        let signal_count = signals.len();
        let coverage_factor = (signal_count as f64 / 15.0).min(1.0);

        // Average signal confidence
        let avg_signal_confidence =
            signals.iter().map(|s| s.confidence).sum::<f64>() / signal_count as f64;

        // Weighted combination
        coverage_factor * 0.4 + avg_signal_confidence * 0.6
    }

    fn generate_recommendations(
        &self,
        signals: &[MaturitySignal],
        category_scores: &CategoryScores,
        current_level: MaturityLevel,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Find low-scoring areas
        if category_scores.git < 0.5 {
            recommendations.push(Recommendation::new(
                "git",
                Priority::High,
                "Improve git practices: add more commits, create branches, or add remote repository",
            ));
        }

        if category_scores.filesystem < 0.5 {
            recommendations.push(Recommendation::new(
                "filesystem",
                Priority::High,
                "Improve project structure: add documentation, tests, and configuration files",
            ));
        }

        if category_scores.environment < 0.5 {
            recommendations.push(Recommendation::new(
                "environment",
                Priority::High,
                "Set up CI/CD, package manager, and linting tools",
            ));
        }

        // Check for specific missing signals
        let signal_names: std::collections::HashSet<String> =
            signals.iter().map(|s| s.name.clone()).collect();

        if !signal_names.contains("test_coverage")
            || signals
                .iter()
                .find(|s| s.name == "test_coverage")
                .map(|s| s.value)
                .unwrap_or(0.0)
                < 0.3
        {
            recommendations.push(Recommendation::new(
                "testing",
                Priority::Medium,
                "Add test files and test configuration",
            ));
        }

        if !signal_names.contains("documentation")
            || signals
                .iter()
                .find(|s| s.name == "documentation")
                .map(|s| s.value)
                .unwrap_or(0.0)
                < 0.5
        {
            recommendations.push(Recommendation::new(
                "documentation",
                Priority::Medium,
                "Add README and LICENSE files",
            ));
        }

        if !signal_names.contains("cicd_config")
            || signals
                .iter()
                .find(|s| s.name == "cicd_config")
                .map(|s| s.value)
                .unwrap_or(0.0)
                < 0.3
        {
            recommendations.push(Recommendation::new(
                "ci/cd",
                Priority::Medium,
                "Set up continuous integration (GitHub Actions, GitLab CI, etc.)",
            ));
        }

        // Level-specific recommendations
        match current_level {
            MaturityLevel::Raw => {
                recommendations.push(Recommendation::new(
                    "general",
                    Priority::Critical,
                    "Project is in early stages. Focus on basic structure and initial commits.",
                ));
            }
            MaturityLevel::Developing => {
                recommendations.push(Recommendation::new(
                    "general",
                    Priority::Medium,
                    "Good progress! Focus on adding more tooling and documentation.",
                ));
            }
            MaturityLevel::Mature => {
                recommendations.push(Recommendation::new(
                    "general",
                    Priority::Low,
                    "Project is mature. Consider adding security scanning and advanced CI features.",
                ));
            }
            MaturityLevel::Established => {
                // No recommendations needed for established projects
            }
        }

        recommendations
    }
}

impl Default for MaturityDecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod engine_tests;
