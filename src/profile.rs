use crate::groups::Group;
use crate::{APIError, Hypothesis, UserAccountID, API_URL};
use color_eyre::Help;
use eyre::WrapErr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Hypothesis {
    /// Fetch profile information for the currently-authenticated user.
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let profile = api.fetch_user_profile()?;
    /// assert!(profile.userid.is_some());
    /// assert_eq!(profile.userid.unwrap(), api.user);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn fetch_user_profile(&self) -> color_eyre::Result<UserProfile> {
        let text = self
            .client
            .get(&format!("{}/profile", API_URL))
            .send()?
            .text()?;
        let result = serde_json::from_str::<UserProfile>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("OutOfCheeseError: Redo from start.");
        Ok(result?)
    }

    /// Fetch the groups for which the currently-authenticated user is a member.
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    /// use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let groups = api.fetch_user_groups()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn fetch_user_groups(&self) -> color_eyre::Result<Vec<Group>> {
        let text = self
            .client
            .get(&format!("{}/profile/groups", API_URL))
            .send()?
            .text()?;
        let result = serde_json::from_str::<Vec<Group>>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("OutOfCheeseError: Redo from start.");
        Ok(result?)
    }
}

/// User profile information
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct UserProfile {
    pub authority: String,
    pub features: HashMap<String, bool>,
    pub preferences: HashMap<String, bool>,
    /// This property will be a string of the format "acct:username@authority" if the request is authenticated.
    /// This property will be null if the request is not authenticated.
    pub userid: Option<UserAccountID>,
}
