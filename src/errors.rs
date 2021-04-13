//! API and CLI specific errors
use std::fmt;

use reqwest::header::InvalidHeaderValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HypothesisError {
    #[error("Make sure input fields are valid")]
    APIError {
        #[source]
        source: APIError,
        raw_text: String
    },
    #[error("Invalid header value")]
    HeaderError(#[from] InvalidHeaderValue),
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("{suggestion:?}")]
    EnvironmentError {
        #[source]
        source: std::env::VarError,
        suggestion: String,
    },
    #[error("JSON format error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Couldn't parse URL")]
    URLError(#[from] url::ParseError),
    #[error("Builder error: {0}")]
    BuilderError(String),
}

/// Errors returned from the Hypothesis API
#[derive(Error, Serialize, Deserialize, Debug, Default, Clone)]
pub struct APIError {
    /// API returned status
    pub status: String,
    /// Cause of failure
    pub reason: String,
}

impl fmt::Display for APIError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status: {}\nReason: {}", self.status, self.reason)
    }
}

#[cfg(feature = "cli")]
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
