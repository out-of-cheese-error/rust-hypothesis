//! [![Crates.io](https://img.shields.io/crates/v/hypothesis.svg)](https://crates.io/crates/hypothesis)
//! [![Docs.rs](https://docs.rs/hypothesis/badge.svg)](https://docs.rs/hypothesis)
//! [![CI](https://github.com/out-of-cheese-error/hypothesis/workflows/Continuous%20Integration/badge.svg)](https://github.com/out-of-cheese-error/hypothesis/actions)
//! [![GitHub release](https://img.shields.io/github/release/out-of-cheese-error/hypothesis.svg)](https://GitHub.com/out-of-cheese-error/hypothesis/releases/)
//! [![dependency status](https://deps.rs/repo/github/out-of-cheese-error/hypothesis/status.svg)](https://deps.rs/repo/github/out-of-cheese-error/hypothesis)
//!
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
//! NOTE: the CLI doesn't currently have all the capabilities of the Rust crate, specifically bulk actions and updating dates are not supported.
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
//! hypothesis = {version = "0.4.0", default-features = false}
//! # For a tokio runtime:
//! tokio = { version = "0.2", features = ["macros"] }
//! ```
//!
//! #### Examples
//! ```rust no_run
//! use hypothesis::Hypothesis;
//! use hypothesis::annotations::{InputAnnotation, Target, Selector};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), hypothesis::errors::HypothesisError> {
//!     let api = Hypothesis::from_env()?;
//!     let new_annotation = InputAnnotation::builder()
//!             .uri("https://www.example.com")
//!             .text("this is a comment")
//!             .target(Target::builder()
//!                .source("https://www.example.com")
//!                .selector(vec![Selector::new_quote("exact text in website to highlight",
//!                                                   "prefix of text",
//!                                                   "suffix of text")])
//!                .build()?)
//!            .tags(vec!["tag1".to_string(), "tag2".to_string()])
//!            .build()?;
//!     api.create_annotation(&new_annotation).await?;
//!     Ok(())
//! }
//! ```
//! See the documentation of the API struct ([`Hypothesis`](https://docs.rs/crate/hypothesis/struct.Hypothesis.html)) for a list of possible queries.
//! Use bulk functions to perform multiple actions - e.g. `api.fetch_annotations` instead of a loop around `api.fetch_annotation`.
//!
//! Check the [documentation](https://docs.rs/crate/hypothesis) for more usage examples.
//!
//! ### Changelog
//! See the [CHANGELOG](CHANGELOG.md)
//!
//! ### Contributing
//! Make sure you have a .env file (added to .gitignore) in the repo root with HYPOTHESIS_NAME, HYPOTHESIS_KEY, and TEST_GROUP_ID
//!
//! ### Caveats / Todo:
//! - Only supports APIKey authorization and hypothes.is authority (i.e. single users).
//! - `Target.selector.RangeSelector` doesn't seem to follow [W3C standards](https://www.w3.org/TR/annotation-model/#range-selector). It's just a hashmap for now.
//! - `Annotation` hypermedia links are stored as a hashmap, b/c I don't know all the possible values.
//! - Need to figure out how `Document` works to properly document it (hah).
//! - Can't delete a group after making it, can leave it though (maybe it's the same thing?)
//! - No idea what `UserProfile.preferences` and `UserProfile.features` mean.
//! - CLI just dumps output as JSON, this is fine right? Fancier CLIs can build on top of this (or use the crate directly)
#[macro_use]
extern crate derive_builder;

use std::collections::HashMap;
use std::str::FromStr;
use std::string::ParseError;
use std::{env, fmt};

use futures::future::try_join_all;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::annotations::{Annotation, InputAnnotation, SearchQuery};
use crate::errors::HypothesisError;
use crate::groups::{Expand, Group, GroupFilters, Member};
use crate::profile::UserProfile;

pub mod annotations;
#[cfg(feature = "cli")]
pub mod cli;
pub mod errors;
pub mod groups;
pub mod profile;

/// Hypothesis API URL
pub const API_URL: &str = "https://api.hypothes.is/api";

/// checks if a variable is the default value of its type
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Hypothesis API client
pub struct Hypothesis {
    /// Authenticated user
    pub username: String,
    /// "acct:{username}@hypothes.is"
    pub user: UserAccountID,
    /// authorized reqwest async client
    client: reqwest::Client,
}

impl Hypothesis {
    /// Make a new Hypothesis client with your username and developer key
    /// (see [here](https://h.readthedocs.io/en/latest/api/authorization/) on how to get one)
    /// # Example
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new(username: &str, developer_key: &str) -> Result<Self, HypothesisError> {
        let user = UserAccountID::from_str(username).expect("This should never error");
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", developer_key))
                .map_err(HypothesisError::HeaderError)?,
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_str("application/vnd.hypothesis.v1+json")
                .map_err(HypothesisError::HeaderError)?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(HypothesisError::ReqwestError)?;
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #    use std::env;
    /// #    dotenv::dotenv()?;
    /// #    let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #    let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #    env::set_var("HYPOTHESIS_NAME", username);
    /// #    env::set_var("HYPOTHESIS_KEY", developer_key);
    /// use hypothesis::Hypothesis;
    /// let api = Hypothesis::from_env()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self, HypothesisError> {
        let username =
            env::var("HYPOTHESIS_NAME").map_err(|e| HypothesisError::EnvironmentError {
                source: e,
                suggestion: "Set the environment variable HYPOTHESIS_NAME to your username".into(),
            })?;
        let developer_key =
            env::var("HYPOTHESIS_KEY").map_err(|e| HypothesisError::EnvironmentError {
                source: e,
                suggestion: "Set the environment variable HYPOTHESIS_KEY to your personal API key"
                    .into(),
            })?;
        Ok(Self::new(&username, &developer_key)?)
    }

    /// Create a new annotation
    ///
    /// Posts a new annotation object to Hypothesis.
    /// Returns an [`Annotation`](annotations/struct.Annotation.html) as output.
    /// See [`InputAnnotation`](annotations/struct.InputAnnotation.html) for examples on what you can add to an annotation.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// use hypothesis::annotations::InputAnnotation;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let annotation = api.create_annotation(&InputAnnotation::builder()
    ///                     .text("string")
    ///                     .uri("http://example.com")
    ///                     .group(&group_id)
    ///                     .build()?).await?;
    /// assert_eq!(&annotation.text, "string");
    /// #    api.delete_annotation(&annotation.id).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn create_annotation(
        &self,
        annotation: &InputAnnotation,
    ) -> Result<Annotation, HypothesisError> {
        let text = self
            .client
            .post(&format!("{}/annotations", API_URL))
            .json(annotation)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Annotation>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Create many new annotations
    ///
    /// Posts multiple new annotation objects asynchronously to Hypothesis.
    /// Returns [`Annotation`](annotations/struct.Annotation.html)s as output.
    /// See [`InputAnnotation`'s](annotations/struct.InputAnnotation.html) docs for examples on what
    /// you can add to an annotation.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use hypothesis::Hypothesis;
    /// # use hypothesis::annotations::InputAnnotation;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let input_annotations = vec![
    ///     InputAnnotation::builder()
    ///         .text("first")
    ///         .uri("http://example.com")
    ///         .group(&group_id)
    ///         .build()?,
    ///     InputAnnotation::builder()
    ///         .text("second")
    ///         .uri("http://example.com")
    ///         .group(&group_id)   
    ///         .build()?
    /// ];
    /// let annotations = api.create_annotations(&input_annotations).await?;
    /// assert_eq!(&annotations[0].text, "first");
    /// assert_eq!(&annotations[1].text, "second");
    /// #    api.delete_annotations(&annotations.into_iter().map(|a| a.id).collect::<Vec<_>>()).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn create_annotations(
        &self,
        annotations: &[InputAnnotation],
    ) -> Result<Vec<Annotation>, HypothesisError> {
        let futures: Vec<_> = annotations
            .iter()
            .map(|a| self.create_annotation(a))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Update an existing annotation
    ///
    /// Change any field in an existing annotation. Returns the modified [`Annotation`](annotations/struct.Annotation.html)
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// use hypothesis::annotations::InputAnnotation;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let mut annotation = api.create_annotation(&InputAnnotation::builder()
    ///                   .text("string")
    ///                   .uri("http://example.com")
    ///                   .tags(vec!["tag1".to_string(), "tag2".to_string()])
    ///                   .group(&group_id)
    ///                   .build()?).await?;
    /// annotation.text = String::from("New String");
    /// let updated_annotation = api.update_annotation(&annotation).await?;
    /// assert_eq!(updated_annotation.id, annotation.id);
    /// assert_eq!(&updated_annotation.text, "New String");
    /// #    api.delete_annotation(&updated_annotation.id).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn update_annotation(
        &self,
        annotation: &Annotation,
    ) -> Result<Annotation, HypothesisError> {
        let text = self
            .client
            .patch(&format!("{}/annotations/{}", API_URL, annotation.id))
            .json(&annotation)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Annotation>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Update many annotations at once
    pub async fn update_annotations(
        &self,
        annotations: &[Annotation],
    ) -> Result<Vec<Annotation>, HypothesisError> {
        let futures: Vec<_> = annotations
            .iter()
            .map(|a| self.update_annotation(&a))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Search for annotations with optional filters
    ///
    /// Returns a list of annotations matching the search query.
    /// See  [`SearchQuery`](annotations/struct.SearchQuery.html) for more filtering options
    ///
    /// This returns a max of 50 annotations at once, use `search_annotations_return_all` if you expect more
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::{Hypothesis, UserAccountID};
    /// use hypothesis::annotations::SearchQuery;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// /// Search for your own annotations:
    /// let search_query = SearchQuery::builder().user(&api.user.0).build()?;
    /// let search_results = api.search_annotations(&search_query).await?;
    /// #     assert!(!search_results.is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn search_annotations(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<Annotation>, HypothesisError> {
        let query: HashMap<String, serde_json::Value> = serde_json::from_str(
            &serde_json::to_string(&query).map_err(HypothesisError::SerdeError)?,
        )
        .map_err(HypothesisError::SerdeError)?;
        let url = Url::parse_with_params(
            &format!("{}/search", API_URL),
            &query
                .into_iter()
                .map(|(k, v)| (k, v.to_string().replace('"', "")))
                .collect::<Vec<_>>(),
        )
        .map_err(HypothesisError::URLError)?;
        let text = self
            .client
            .get(url)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        #[derive(Deserialize, Debug, Clone, PartialEq)]
        struct SearchResult {
            rows: Vec<Annotation>,
            total: usize,
        }
        let result = serde_json::from_str::<SearchResult>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?.rows)
    }

    /// Retrieve all annotations matching query
    /// See  [`SearchQuery`](annotations/struct.SearchQuery.html) for filtering options
    pub async fn search_annotations_return_all(
        &self,
        query: &mut SearchQuery,
    ) -> Result<Vec<Annotation>, HypothesisError> {
        let mut annotations = Vec::new();
        loop {
            let next = self.search_annotations(query).await?;
            if next.is_empty() {
                break;
            }
            query.search_after = next[next.len() - 1].updated.to_rfc3339();
            annotations.extend_from_slice(&next);
        }
        Ok(annotations)
    }

    /// Fetch annotation by ID
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #    use hypothesis::annotations::InputAnnotation;
    /// #    dotenv::dotenv()?;
    /// #    let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #    let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// #    let annotation = api.create_annotation(&InputAnnotation::builder()
    /// #                       .text("string")
    /// #                       .uri("http://example.com")
    /// #                       .group(group_id).build()?).await?;
    /// #    let annotation_id = annotation.id.to_owned();    
    /// let annotation = api.fetch_annotation(&annotation_id).await?;
    /// assert_eq!(annotation.id, annotation_id);
    /// #    api.delete_annotation(&annotation.id).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn fetch_annotation(&self, id: &str) -> Result<Annotation, HypothesisError> {
        let text = self
            .client
            .get(&format!("{}/annotations/{}", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Annotation>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Fetch multiple annotations by ID
    pub async fn fetch_annotations(
        &self,
        ids: &[String],
    ) -> Result<Vec<Annotation>, HypothesisError> {
        let futures: Vec<_> = ids.iter().map(|id| self.fetch_annotation(id)).collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Delete annotation by ID
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #    use hypothesis::annotations::InputAnnotation;
    /// #    dotenv::dotenv()?;
    /// #    let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #    let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// #    let annotation = api.create_annotation(&InputAnnotation::builder()
    /// #                       .text("string")
    /// #                       .uri("http://example.com")
    /// #                       .group(group_id).build()?).await?;
    /// #    let annotation_id = annotation.id.to_owned();    
    /// let deleted = api.delete_annotation(&annotation_id).await?;
    /// assert!(deleted);
    /// assert!(api.fetch_annotation(&annotation_id).await.is_err());
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn delete_annotation(&self, id: &str) -> Result<bool, HypothesisError> {
        let text = self
            .client
            .delete(&format!("{}/annotations/{}", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        #[derive(Deserialize, Debug, Clone, PartialEq)]
        struct DeletionResult {
            id: String,
            deleted: bool,
        }
        let result = serde_json::from_str::<DeletionResult>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?.deleted)
    }

    /// Delete multiple annotations by ID
    pub async fn delete_annotations(&self, ids: &[String]) -> Result<Vec<bool>, HypothesisError> {
        let futures: Vec<_> = ids.iter().map(|id| self.delete_annotation(id)).collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Flag an annotation
    ///
    /// Flag an annotation for review (moderation). The moderator of the group containing the
    /// annotation will be notified of the flag and can decide whether or not to hide the
    /// annotation. Note that flags persist and cannot be removed once they are set.
    pub async fn flag_annotation(&self, id: &str) -> Result<(), HypothesisError> {
        let text = self
            .client
            .put(&format!("{}/annotations/{}/flag", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let error = serde_json::from_str::<errors::APIError>(&text);
        if let Ok(error) = error {
            Err(HypothesisError::APIError(error))
        } else {
            Ok(())
        }
    }

    /// Hide an annotation
    ///
    /// Hide an annotation. The authenticated user needs to have the moderate permission for the
    /// group that contains the annotation — this permission is granted to the user who created the group.
    pub async fn hide_annotation(&self, id: &str) -> Result<(), HypothesisError> {
        let text = self
            .client
            .put(&format!("{}/annotations/{}/hide", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let error = serde_json::from_str::<errors::APIError>(&text);
        if let Ok(error) = error {
            Err(HypothesisError::APIError(error))
        } else {
            Ok(())
        }
    }

    /// Show an annotation
    ///
    /// Show/"un-hide" an annotation. The authenticated user needs to have the moderate permission
    /// for the group that contains the annotation—this permission is granted to the user who created the group.
    pub async fn show_annotation(&self, id: &str) -> Result<(), HypothesisError> {
        let text = self
            .client
            .delete(&format!("{}/annotations/{}/hide", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let error = serde_json::from_str::<errors::APIError>(&text);
        if let Ok(error) = error {
            Err(HypothesisError::APIError(error))
        } else {
            Ok(())
        }
    }

    /// Retrieve a list of applicable Groups, filtered by authority and target document (`document_uri`).
    /// Also retrieve user's private Groups.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// use hypothesis::groups::GroupFilters;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    ///
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// /// Get all Groups belonging to user
    /// let groups = api.get_groups(&GroupFilters::default()).await?;
    /// #    assert!(!groups.is_empty());
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn get_groups(&self, query: &GroupFilters) -> Result<Vec<Group>, HypothesisError> {
        let query: HashMap<String, serde_json::Value> = serde_json::from_str(
            &serde_json::to_string(&query).map_err(HypothesisError::SerdeError)?,
        )
        .map_err(HypothesisError::SerdeError)?;
        let url = Url::parse_with_params(
            &format!("{}/groups", API_URL),
            &query
                .into_iter()
                .map(|(k, v)| (k, v.to_string().replace('"', "")))
                .collect::<Vec<_>>(),
        )
        .map_err(HypothesisError::URLError)?;
        let text = self
            .client
            .get(url)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Vec<Group>>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Create a new, private group for the currently-authenticated user.
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    ///
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let group = api.create_group("my_group", Some("a test group")).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn create_group(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<Group, HypothesisError> {
        let mut params = HashMap::new();
        params.insert("name", name);
        if let Some(description) = description {
            params.insert("description", description);
        }
        let text = self
            .client
            .post(&format!("{}/groups", API_URL))
            .json(&params)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Group>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Create multiple groups
    pub async fn create_groups(
        &self,
        names: &[String],
        descriptions: &[Option<String>],
    ) -> Result<Vec<Group>, HypothesisError> {
        let futures: Vec<_> = names
            .iter()
            .zip(descriptions.iter())
            .map(|(name, description)| self.create_group(name, description.as_deref()))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Fetch a single Group resource.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// use hypothesis::groups::Expand;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID")?;
    ///
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// /// Expands organization into a struct
    /// let group = api.fetch_group(&group_id, vec![Expand::Organization]).await?;
    /// #    Ok(())
    /// # }    
    /// ```
    pub async fn fetch_group(
        &self,
        id: &str,
        expand: Vec<Expand>,
    ) -> Result<Group, HypothesisError> {
        let params: HashMap<&str, Vec<String>> = if !expand.is_empty() {
            vec![(
                "expand",
                expand
                    .into_iter()
                    .map(|e| serde_json::to_string(&e))
                    .collect::<Result<_, _>>()
                    .map_err(HypothesisError::SerdeError)?,
            )]
            .into_iter()
            .collect()
        } else {
            HashMap::new()
        };
        let text = self
            .client
            .get(&format!("{}/groups/{}", API_URL, id))
            .json(&params)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Group>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Fetch multiple groups by ID
    pub async fn fetch_groups(
        &self,
        ids: &[String],
        expands: Vec<Vec<Expand>>,
    ) -> Result<Vec<Group>, HypothesisError> {
        let futures: Vec<_> = ids
            .iter()
            .zip(expands.into_iter())
            .map(|(id, expand)| self.fetch_group(id, expand))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Update a Group resource.
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID")?;
    ///
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let group = api.update_group(&group_id, Some("new_group_name"), None).await?;
    /// assert_eq!(&group.name, "new_group_name");
    /// assert_eq!(group.id, group_id);
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn update_group(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Group, HypothesisError> {
        let mut params = HashMap::new();
        if let Some(name) = name {
            params.insert("name", name);
        }
        if let Some(description) = description {
            params.insert("description", description);
        }
        let text = self
            .client
            .patch(&format!("{}/groups/{}", API_URL, id))
            .json(&params)
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Group>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Update multiple groups
    pub async fn update_groups(
        &self,
        ids: &[String],
        names: &[Option<String>],
        descriptions: &[Option<String>],
    ) -> Result<Vec<Group>, HypothesisError> {
        let futures: Vec<_> = ids
            .iter()
            .zip(names.iter())
            .zip(descriptions.iter())
            .map(|((id, name), description)| {
                self.update_group(id, name.as_deref(), description.as_deref())
            })
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Fetch a list of all members (users) in a group. Returned user resource only contains public-facing user data.
    /// Authenticated user must have read access to the group. Does not require authentication for reading members of
    /// public groups. Returned members are unsorted.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID")?;
    ///
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let members = api.get_group_members(&group_id).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn get_group_members(&self, id: &str) -> Result<Vec<Member>, HypothesisError> {
        let text = self
            .client
            .get(&format!("{}/groups/{}/members", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Vec<Member>>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Remove yourself from a group.
    pub async fn leave_group(&self, id: &str) -> Result<(), HypothesisError> {
        let text = self
            .client
            .delete(&format!("{}/groups/{}/members/me", API_URL, id))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let error = serde_json::from_str::<errors::APIError>(&text);
        if let Ok(error) = error {
            Err(HypothesisError::APIError(error))
        } else {
            Ok(())
        }
    }

    /// Fetch profile information for the currently-authenticated user.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let profile = api.fetch_user_profile().await?;
    /// assert!(profile.userid.is_some());
    /// assert_eq!(profile.userid.unwrap(), api.user);
    /// #     Ok(())
    /// # }
    /// ```

    pub async fn fetch_user_profile(&self) -> Result<UserProfile, HypothesisError> {
        let text = self
            .client
            .get(&format!("{}/profile", API_URL))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<UserProfile>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }

    /// Fetch the groups for which the currently-authenticated user is a member.
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("HYPOTHESIS_NAME")?;
    /// #     let developer_key = dotenv::var("HYPOTHESIS_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let groups = api.fetch_user_groups().await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn fetch_user_groups(&self) -> Result<Vec<Group>, HypothesisError> {
        let text = self
            .client
            .get(&format!("{}/profile/groups", API_URL))
            .send()
            .await
            .map_err(HypothesisError::ReqwestError)?
            .text()
            .await
            .map_err(HypothesisError::ReqwestError)?;
        let result = serde_json::from_str::<Vec<Group>>(&text).map_err(|_| {
            HypothesisError::APIError(
                serde_json::from_str::<errors::APIError>(&text).unwrap_or_default(),
            )
        });
        Ok(result?)
    }
}

/// Stores user account ID in the form "acct:{username}@hypothes.is"
///
/// Create from username:
/// ```
/// # use hypothesis::UserAccountID;
/// let user_id = "my_username".parse::<UserAccountID>().unwrap();
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserAccountID(pub String);

impl FromStr for UserAccountID {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(format!("acct:{}@hypothes.is", s)))
    }
}

impl fmt::Display for UserAccountID {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&UserAccountID> for UserAccountID {
    #[inline]
    fn from(a: &UserAccountID) -> UserAccountID {
        UserAccountID(a.0.to_owned())
    }
}
