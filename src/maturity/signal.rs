use serde::{Deserialize, Serialize};
use std::path::Path;

use super::MaturityResult;

/// Types of signals that can be collected for maturity detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Git-related signals (repository age, commit history, etc.)
    Git,
    /// File system signals (file count, directory structure, etc.)
    Filesystem,
    /// Environment signals (CI/CD, package managers, tools)
    Environment,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::Git => write!(f, "git"),
            SignalType::Filesystem => write!(f, "filesystem"),
            SignalType::Environment => write!(f, "environment"),
        }
    }
}

/// Represents a single maturity signal with its value and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaturitySignal {
    /// The type of signal
    pub signal_type: SignalType,
    /// Signal name/identifier
    pub name: String,
    /// Numeric value of the signal (normalized to 0.0-1.0 where applicable)
    pub value: f64,
    /// Raw value before normalization (for display/debugging)
    pub raw_value: String,
    /// Confidence level of this signal (0.0-1.0)
    pub confidence: f64,
    /// Weight of this signal in overall scoring
    pub weight: f64,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl MaturitySignal {
    pub fn new(
        signal_type: SignalType,
        name: impl Into<String>,
        value: f64,
        raw_value: impl Into<String>,
    ) -> Self {
        Self {
            signal_type,
            name: name.into(),
            value: value.clamp(0.0, 1.0),
            raw_value: raw_value.into(),
            confidence: 1.0,
            weight: 1.0,
            metadata: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight.max(0.0);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get the weighted contribution of this signal
    pub fn weighted_value(&self) -> f64 {
        self.value * self.weight * self.confidence
    }
}

/// Trait for signal collectors
pub trait SignalCollector: Send + Sync {
    /// Get the type of signals this collector produces
    fn signal_type(&self) -> SignalType;

    /// Collect signals from the given path
    fn collect(&self, path: &Path) -> MaturityResult<Vec<MaturitySignal>>;

    /// Check if this collector can handle the given path
    fn can_collect(&self, path: &Path) -> bool {
        path.exists()
    }
}

/// Signal collection pipeline for managing multiple collectors
#[derive(Default)]
pub struct SignalPipeline {
    collectors: Vec<Box<dyn SignalCollector>>,
}

impl SignalPipeline {
    pub fn new() -> Self {
        Self {
            collectors: Vec::new(),
        }
    }

    pub fn add_collector(mut self, collector: Box<dyn SignalCollector>) -> Self {
        self.collectors.push(collector);
        self
    }

    pub fn collect_all(&self, path: &Path) -> MaturityResult<Vec<MaturitySignal>> {
        let mut all_signals = Vec::new();

        for collector in &self.collectors {
            if collector.can_collect(path) {
                match collector.collect(path) {
                    Ok(signals) => all_signals.extend(signals),
                    Err(e) => {
                        tracing::warn!(
                            "Collector {} failed: {}",
                            collector.signal_type(),
                            e
                        );
                    }
                }
            }
        }

        Ok(all_signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = MaturitySignal::new(SignalType::Git, "test_signal", 0.5, "50");
        assert_eq!(signal.signal_type, SignalType::Git);
        assert_eq!(signal.name, "test_signal");
        assert_eq!(signal.value, 0.5);
        assert_eq!(signal.raw_value, "50");
        assert_eq!(signal.confidence, 1.0);
        assert_eq!(signal.weight, 1.0);
    }

    #[test]
    fn test_signal_with_confidence() {
        let signal = MaturitySignal::new(SignalType::Filesystem, "test", 0.8, "80")
            .with_confidence(0.9)
            .with_weight(2.0);

        assert_eq!(signal.confidence, 0.9);
        assert_eq!(signal.weight, 2.0);
        assert_eq!(signal.weighted_value(), 0.8 * 0.9 * 2.0);
    }

    #[test]
    fn test_signal_value_clamping() {
        let signal = MaturitySignal::new(SignalType::Environment, "test", 1.5, "150");
        assert_eq!(signal.value, 1.0);

        let signal = MaturitySignal::new(SignalType::Environment, "test", -0.5, "-50");
        assert_eq!(signal.value, 0.0);
    }
}
