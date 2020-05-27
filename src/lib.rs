pub mod annotations;
pub mod groups;
pub mod profile;

use reqwest::header;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub const API_URL: &str = "https://api.hypothes.is/api";
pub type GroupID = String;
pub type AnnotationID = String;
pub type UserAccountID = String;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Hypothesis API client
pub struct Hypothesis {
    /// Authenticated user
    pub username: String,
    /// "acct:<username>@hypothes.is"
    pub user: UserAccountID,
    client: reqwest::blocking::Client,
}

impl Hypothesis {
    /// Make a new Hypothesis client with your username and developer key
    /// (see [here](https://h.readthedocs.io/en/latest/api/authorization/) on how to get one)
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new(username: &str, developer_key: &str) -> color_eyre::Result<Self> {
        let user = format!("acct:{}@hypothes.is", username);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", developer_key))?,
        );
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Self {
            username: username.into(),
            user,
            client,
        })
    }
}

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
