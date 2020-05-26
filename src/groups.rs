use crate::{is_default, GroupID, Hypothesis, API_URL};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Hypothesis {
    /// Retrieve a list of applicable Groups, filtered by authority and target document (document_uri).
    /// Also retrieve user's private Groups.
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    ///     use hypothesis::groups::GroupFilters;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    ///
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     /// Get all Groups belonging to user
    ///     let groups = api.get_groups(&GroupFilters::default())?;
    ///     assert!(!groups.is_empty());
    /// #    Ok(())
    /// # }
    /// ```
    pub fn get_groups(&self, query: &GroupFilters) -> color_eyre::Result<Vec<Group>> {
        let query: HashMap<String, serde_json::Value> =
            serde_json::from_str(&serde_json::to_string(&query)?)?;
        let url = Url::parse_with_params(
            &format!("{}/groups", API_URL),
            &query
                .into_iter()
                .map(|(k, v)| (k, v.to_string().replace('"', "")))
                .collect::<Vec<_>>(),
        )?;
        Ok(self.client.get(url).send()?.json()?)
    }

    /// Create a new, private group for the currently-authenticated user.
    ///
    /// # Example
    /// ```no_run
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    ///
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let group = api.create_group("my_group", Some("a test group"))?;
    /// #    
    /// #    Ok(())
    /// # }
    /// ```
    pub fn create_group(&self, name: &str, description: Option<&str>) -> color_eyre::Result<Group> {
        let mut params = HashMap::new();
        params.insert("name", name);
        if let Some(description) = description {
            params.insert("description", description);
        }
        Ok(self
            .client
            .post(&format!("{}/groups", API_URL))
            .json(&params)
            .send()?
            .json()?)
    }

    /// Fetch a single Group resource.
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// use hypothesis::groups::Expand;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID")?;
    ///
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     /// Expands organization into a struct
    ///     let group = api.fetch_group(&group_id, vec![Expand::Organization])?;
    /// #    
    /// #    Ok(())
    /// # }    
    /// ```
    pub fn fetch_group(&self, id: &GroupID, expand: Vec<Expand>) -> color_eyre::Result<Group> {
        let params: HashMap<&str, Vec<String>> = if !expand.is_empty() {
            vec![(
                "expand",
                expand
                    .into_iter()
                    .map(|e| serde_json::to_string(&e))
                    .collect::<Result<_, _>>()?,
            )]
            .into_iter()
            .collect()
        } else {
            HashMap::new()
        };
        Ok(self
            .client
            .get(&format!("{}/groups/{}", API_URL, id))
            .json(&params)
            .send()?
            .json()?)
    }

    /// Update a Group resource.
    ///
    /// # Example
    /// ```no_run
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID")?;
    ///
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let group = api.update_group(&group_id, Some("new_group_name"), None)?;
    ///     assert_eq!(&group.name, "new_group_name");
    ///     assert_eq!(group.id, group_id);
    /// #    Ok(())
    /// # }
    /// ```
    pub fn update_group(
        &self,
        id: &GroupID,
        name: Option<&str>,
        description: Option<&str>,
    ) -> color_eyre::Result<Group> {
        let mut params = vec![];
        if let Some(name) = name {
            params.push(("name", name));
        }
        if let Some(description) = description {
            params.push(("description", description));
        }
        Ok(self
            .client
            .patch(&format!("{}/groups/{}", API_URL, id))
            .form(&params)
            .send()?
            .json()?)
    }

    /// Fetch a list of all members (users) in a group. Returned user resource only contains public-facing user data.
    /// Authenticated user must have read access to the group. Does not require authentication for reading members of
    /// public groups. Returned members are unsorted.
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID")?;
    ///
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let members = api.get_group_members(&group_id)?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn get_group_members(&self, id: &GroupID) -> color_eyre::Result<Vec<GroupMember>> {
        Ok(self
            .client
            .get(&format!("{}/groups/{}/members", API_URL, id))
            .send()?
            .json()?)
    }

    /// Remove yourself from a group.
    pub fn leave_group(&self, id: &GroupID) -> color_eyre::Result<()> {
        self.client
            .delete(&format!("{}/groups/{}/members/me", API_URL, id))
            .send()?;
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Expand {
    Organization,
    Scopes,
}

/// Filter groups by authority and target document
#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct GroupFilters {
    /// Filter returned groups to this authority. For authenticated requests, the user's associated authority will supersede any provided value.
    /// Default: "hypothes.is"
    #[serde(skip_serializing_if = "is_default")]
    pub authority: String,
    /// Only retrieve public (i.e. non-private) groups that apply to a given document URI (i.e. the target document being annotated).
    #[serde(skip_serializing_if = "is_default")]
    pub document_uri: String,
    /// One or more relations to expand for a group resource
    #[serde(skip_serializing_if = "is_default")]
    pub expand: Vec<Expand>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Links {
    /// URL to the group's main (activity) page
    #[serde(default)]
    pub html: Option<String>,
}

/// See [the Hypothesis API docs](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/groups/paths/~1groups/get) for more information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Scope {
    pub enforced: bool,
    pub uri_patterns: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Private,
    Open,
    Restricted,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Organization {
    String(String),
    Organization(Option<Org>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Org {
    pub id: String,
    /// true if this organization is the default organization for the current authority
    pub default: bool,
    /// URI to logo image; may be null if no logo exists
    pub logo: Option<String>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Group {
    /// Group ID
    pub id: GroupID,
    /// Authority-unique identifier that may be set for groups that are owned by a third-party authority.
    /// This field is currently present but unused for first-party-authority groups.
    pub groupid: Option<GroupID>,
    /// Group name
    pub name: String,
    pub links: Links,
    /// The organization to which this group belongs.
    pub organization: Organization,
    #[serde(default)]
    /// Information about the URL restrictions for annotations within this group.
    pub scopes: Option<Scope>,
    /// Whether or not this group has URL restrictions for documents that may be annotated within it.
    /// Non-scoped groups allow annotation to documents at any URL
    pub scoped: bool,
    #[serde(rename = "type")]
    pub group_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GroupMember {
    pub authority: String,
    /// // string [ 3 .. 30 ] characters ^[A-Za-z0-9._]+$
    pub username: String,
    /// string^acct:.+$
    pub userid: String,
    /// string <= 30 characters
    pub display_name: String,
}
