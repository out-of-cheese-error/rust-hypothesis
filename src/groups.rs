use crate::{is_default, AnnotationID, GroupID, Hypothesis, UserAccountID, API_URL};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Hypothesis {
    pub fn get_groups(&self, query: &GroupListQuery) -> color_eyre::Result<Vec<Group>> {
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
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Expand {
    Organization,
    Scopes,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct GroupListQuery {
    #[serde(skip_serializing_if = "is_default")]
    pub authority: String,
    #[serde(skip_serializing_if = "is_default")]
    pub document_uri: String,
    #[serde(skip_serializing_if = "is_default")]
    pub expand: Vec<Expand>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Links {
    #[serde(default)]
    pub html: Option<String>,
}

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
pub struct Group {
    pub id: String,
    pub groupid: Option<GroupID>,
    pub name: String,
    pub links: Links,
    #[serde(default)]
    pub scopes: Option<Scope>,
    pub scoped: bool,
    #[serde(rename = "type")]
    pub group_type: Type,
}
