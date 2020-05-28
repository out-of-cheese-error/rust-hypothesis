use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors returned from the Hypothesis API
#[derive(Error, Serialize, Deserialize, Debug, Default, Clone)]
pub struct APIError {
    pub status: String,
    pub reason: String,
}

impl fmt::Display for APIError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status: {}\nReason: {}", self.status, self.reason)
    }
}

#[cfg(feature = "application")]
/// Errors returned from the Hypothesis CLI
#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum CLIError {
    /// Thrown when Hypothesis client creation fails
    #[error("Could not authorize")]
    AuthorizationError,
    /// Failed to parse a command line argument into its corresponding type
    #[error("ParseError: {name:?} must be one of {types:?}")]
    ParseError { name: String, types: Vec<String> },
}
