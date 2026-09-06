//! Experimental local observations and manually selected constraint tiers.

pub mod level;
pub mod observations;

pub use level::MaturityLevel;
pub use observations::{CiExecutionState, ProjectObservations};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaturityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type MaturityResult<T> = Result<T, MaturityError>;
