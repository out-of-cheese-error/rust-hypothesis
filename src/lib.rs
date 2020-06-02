//! [![Crates.io](https://img.shields.io/crates/v/hypothesis.svg)](https://crates.io/crates/hypothesis)
//! # A Rust API for [Hypothesis](https://web.hypothes.is/)
//!
//! ## Description
//! A lightweight wrapper and CLI for the [Hypothesis Web API v1.0.0](https://h.readthedocs.io/en/latest/api-reference/v1/).
//! It includes all APIKey authorized endpoints related to
//! * annotations (create / update / delete / search / fetch / flag),
//! * groups (create / update / list / fetch / leave / members)
//! * profile (user information / groups)
//!
//! ## Installation and Usage
//! ### Authorization
//! You'll need a [Hypothesis](https://hypothes.is) account, and a personal API token obtained as described [here](https://h.readthedocs.io/en/latest/api/authorization/).
//! Set the environment variables `$HYPOTHESIS_NAME` and `$HYPOTHESIS_KEY` to your username and the developer API key respectively.
//!
//! ### As a command-line utility:
//! ```bash
//! cargo install hypothesis
//! ```
//! Run `hypothesis --help` to see subcommands and options.
//!
//! Generate shell completions:
//! ```bash
//! hypothesis complete zsh > .oh-my-zsh/completions/_hypothesis
//! exec zsh
//! ```
//!
//! ### As a Rust crate
//! Add to your Cargo.toml:
//! ```toml
//! [dependencies]
//! hypothesis = {version = "0.3.0", default-features = false}
//! # For a tokio runtime:
//! tokio = { version = "0.2", features = ["macros"] }
//! ```
//!
//! #### Examples
//! ```rust no_run
//! use hypothesis::Hypothesis;
//! use hypothesis::annotations::{InputAnnotationBuilder, TargetBuilder, Selector, TextQuoteSelector};
//!
//! #[tokio::main]
//! async fn main() -> color_eyre::Result<()> {
//!    let api = Hypothesis::from_env()?;
//!    let new_annotation = api.create_annotation(
//!         &InputAnnotationBuilder::default()
//!             .uri("https://www.example.com")
//!             .text("this is a comment")
//!             .target(TargetBuilder::default()
//!                .source("https://www.example.com")
//!                .selector(vec![Selector::new_quote("exact text in website to highlight",
//!                                                   "prefix of text",
//!                                                   "suffix of text")])
//!                .build()?)
//!            .tags(vec!["tag1".to_string(), "tag2".to_string()])
//!            .build()?
//!    ).await?;
//!    Ok(())
//! }
//! ```
//! Use bulk functions to perform multiple actions - e.g. `api.fetch_annotations` instead of a
//! loop around `api.fetch_annotation`.
//!
//! Check the [documentation](https://docs.rs/crate/hypothesis) for more usage examples.
//!
//! ### Changelog
//! See the [CHANGELOG](CHANGELOG.md)
//!
//! ### Caveats / Todo:
//! - ~~Blocking API (nothing stopping async except my lack of experience with it though)~~ Async from v0.3!.
//! - Only supports APIKey authorization and hypothes.is authority (i.e. single users).
//! - `Target.selector.RangeSelector` doesn't seem to follow [W3C standards](https://www.w3.org/TR/annotation-model/#range-selector). It's just a hashmap for now.
//! - `Annotation` hypermedia links are stored as a hashmap, b/c I don't know all the possible values.
//! - Need to figure out how `Document` works to properly document it (hah).
//! - Can't delete a group after making it, can leave it though (maybe it's the same thing?)
//! - No idea what `UserProfile.preferences` and `UserProfile.features` mean.
//! - CLI just dumps output as JSON, this is fine right? Fancier CLIs can build on top of this (or use the crate directly)

#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate eyre;

use std::str::FromStr;
use std::string::ParseError;
use std::{env, fmt};

use color_eyre::Help;
use reqwest::header;
use serde::{Deserialize, Serialize};

pub mod annotations;
pub mod bulk;
#[cfg(feature = "cli")]
pub mod cli;
pub mod errors;
pub mod groups;
pub mod profile;

pub const API_URL: &str = "https://api.hypothes.is/api";

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

impl Into<UserAccountID> for &UserAccountID {
    fn into(self) -> UserAccountID {
        UserAccountID(self.0.to_owned())
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
