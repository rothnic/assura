pub mod config;
pub mod engine;
pub mod environment;
pub mod filesystem;
#[cfg(feature = "git-signals")]
pub mod git;
pub mod signal;

pub use config::MaturityConfig;
pub use engine::{MaturityDecisionEngine, MaturityLevel, MaturityReport, Priority, Recommendation};
pub use environment::EnvironmentSignals;
pub use filesystem::FilesystemSignals;
#[cfg(feature = "git-signals")]
pub use git::GitSignals;
pub use signal::{MaturitySignal, SignalCollector, SignalPipeline, SignalType};

use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaturityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Signal collection error: {0}")]
    SignalCollection(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type MaturityResult<T> = Result<T, MaturityError>;

/// Main entry point for maturity detection
pub struct MaturityDetector {
    collectors: Vec<Box<dyn SignalCollector>>,
    engine: MaturityDecisionEngine,
    config: Option<MaturityConfig>,
}

impl MaturityDetector {
    pub fn new() -> Self {
        #[cfg(feature = "git-signals")]
        let collectors: Vec<Box<dyn SignalCollector>> = vec![
            Box::new(GitSignals::new()),
            Box::new(FilesystemSignals::new()),
            Box::new(EnvironmentSignals::new()),
        ];

        #[cfg(not(feature = "git-signals"))]
        let collectors: Vec<Box<dyn SignalCollector>> = vec![
            Box::new(FilesystemSignals::new()),
            Box::new(EnvironmentSignals::new()),
        ];

        Self {
            collectors,
            engine: MaturityDecisionEngine::new(),
            config: None,
        }
    }

    pub fn with_config(mut self, config: MaturityConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn detect<P: AsRef<Path>>(&self, path: P) -> MaturityResult<MaturityReport> {
        let path = path.as_ref();

        // Check for config override first
        if let Some(ref config) = self.config {
            if config.is_manual_override() {
                return Ok(config.to_report());
            }
        }

        // Collect signals from all collectors
        let mut all_signals = Vec::new();
        for collector in &self.collectors {
            match collector.collect(path) {
                Ok(signals) => all_signals.extend(signals),
                Err(e) => {
                    tracing::warn!("Signal collection failed: {}", e);
                }
            }
        }

        // Apply decision engine
        let mut report = self.engine.evaluate(&all_signals);

        // Apply config adjustments if present
        if let Some(ref config) = self.config {
            report = config.adjust_report(report);
        }

        Ok(report)
    }
}

impl Default for MaturityDetector {
    fn default() -> Self {
        Self::new()
    }
}
