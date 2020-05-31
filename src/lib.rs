pub mod annotations;
pub mod bulk;
#[cfg(feature = "cli")]
pub mod cli;
pub mod errors;
pub mod groups;
pub mod profile;

use color_eyre::Help;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::string::ParseError;
use std::{env, fmt};

pub const API_URL: &str = "https://api.hypothes.is/api";
pub type GroupID = String;
pub type AnnotationID = String;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserAccountID(String);

impl FromStr for UserAccountID {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl UserAccountID {
    pub fn from(username: &str) -> Self {
        Self(format!("acct:{}@hypothes.is", username))
    }

    pub fn get(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for UserAccountID {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserID: {}", self.0)
    }
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Hypothesis API client
pub struct Hypothesis {
    /// Authenticated user
    pub username: String,
    /// "acct:{username}@hypothes.is"
    pub user: UserAccountID,
    client: reqwest::Client,
}

impl Hypothesis {
    /// Make a new Hypothesis client with your username and developer key
    /// (see [here](https://h.readthedocs.io/en/latest/api/authorization/) on how to get one)
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new(username: &str, developer_key: &str) -> color_eyre::Result<Self> {
        let user = UserAccountID::from_str(username)?;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", developer_key))?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Self {
            username: username.into(),
            user,
            client,
        })
    }

    /// Make a new Hypothesis client from environment variables.
    /// Username from `$HYPOTHESIS_NAME`,
    /// Developer key from `$HYPOTHESIS_KEY`
    /// (see [here](https://h.readthedocs.io/en/latest/api/authorization/) on how to get one)
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    /// #    use std::env;
    /// #    dotenv::dotenv()?;
    /// #    let username = dotenv::var("USERNAME")?;
    /// #    let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #    env::set_var("HYPOTHESIS_NAME", username);
    /// #    env::set_var("HYPOTHESIS_KEY", developer_key);
    /// use hypothesis::Hypothesis;
    /// let api = Hypothesis::from_env()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn from_env() -> color_eyre::Result<Self> {
        let username = env::var("HYPOTHESIS_NAME")
            .suggestion("Set the environment variable HYPOTHESIS_NAME to your username")?;
        let developer_key = env::var("HYPOTHESIS_KEY")
            .suggestion("Set the environment variable HYPOTHESIS_KEY to your personal API key")?;
        Ok(Self::new(&username, &developer_key)?)
    }
}
