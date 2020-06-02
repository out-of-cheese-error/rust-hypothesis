//! Objects related to the "profile" endpoint

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::UserAccountID;

/// User profile information
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct UserProfile {
    /// "hypothes.is"
    pub authority: String,
    pub features: HashMap<String, bool>,
    pub preferences: HashMap<String, bool>,
    /// This property will be a string of the format "acct:username@authority" if the request is authenticated.
    /// This property will be null if the request is not authenticated.
    pub userid: Option<UserAccountID>,
}
