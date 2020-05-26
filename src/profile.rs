use crate::groups::Group;
use crate::{Hypothesis, UserAccountID, API_URL};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Hypothesis {
    /// Fetch profile information for the currently-authenticated user.
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let profile = api.fetch_user_profile()?;
    ///     assert!(profile.userid.is_some());
    ///     assert_eq!(profile.userid.unwrap(), api.user);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn fetch_user_profile(&self) -> color_eyre::Result<UserProfile> {
        Ok(self
            .client
            .get(&format!("{}/profile", API_URL))
            .send()?
            .json()?)
    }

    /// Fetch the groups for which the currently-authenticated user is a member.
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let groups = api.fetch_user_groups()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn fetch_user_groups(&self) -> color_eyre::Result<Vec<Group>> {
        Ok(self
            .client
            .get(&format!("{}/profile/groups", API_URL))
            .send()?
            .json()?)
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
