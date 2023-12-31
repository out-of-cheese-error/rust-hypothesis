//! Objects related to the "groups" endpoint

#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::is_default;

/// Which field to expand
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Expand {
    /// Expand `organization` field to `Org`
    Organization,
    /// Expand `scopes` field to `Scope`
    Scopes,
}

/// Filter groups by authority and target document
#[cfg_attr(feature = "cli", derive(Parser))]
#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct GroupFilters {
    /// Filter returned groups to this authority.
    /// For authenticated requests, the user's associated authority will supersede any provided value.
    ///
    /// Default: "hypothes.is"
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", clap(default_value = "hypothes.is", long))]
    pub authority: String,
    /// Only retrieve public (i.e. non-private) groups that apply to a given document URI (i.e. the target document being annotated).
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", clap(default_value = "", long))]
    pub document_uri: String,
    /// One or more relations to expand for a group resource.
    /// Possible values: organization, scopes
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", clap(long, value_parser = clap::builder::EnumValueParser::<Expand>::new()))]
    pub expand: Vec<Expand>,
}

/// URL to the group's main (activity) page
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

/// Group type
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    /// Only creator can view and edit
    Private,
    /// Anyone can view and edit
    Open,
    /// More than one user can view and edit
    Restricted,
}

/// Information about an organization
/// Can be just the organization ID,
/// an `Org` struct,
/// or None if user is not authorized to access this organization
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Organization {
    /// Unexpanded = Unique organization ID
    String(String),
    /// Expanded (None if not authorized)
    Organization(Option<Org>),
}

/// Information about an organization
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Org {
    /// Organization ID
    pub id: String,
    /// true if this organization is the default organization for the current authority
    pub default: bool,
    /// URI to logo image; may be null if no logo exists
    pub logo: Option<String>,
    /// Organization name
    pub name: String,
}

/// Information returned about a Group resource
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Group {
    /// Group ID
    pub id: String,
    /// Authority-unique identifier that may be set for groups that are owned by a third-party authority.
    /// This field is currently present but unused for first-party-authority groups.
    pub groupid: Option<String>,
    /// Group name
    pub name: String,
    /// URL to the group's main (activity) page
    pub links: Links,
    /// The organization to which this group belongs.
    pub organization: Organization,
    #[serde(default)]
    /// Information about the URL restrictions for annotations within this group.
    pub scopes: Option<Scope>,
    /// Whether or not this group has URL restrictions for documents that may be annotated within it.
    /// Non-scoped groups allow annotation to documents at any URL
    pub scoped: bool,
    /// Is the groyp private, open, or restricted
    #[serde(rename = "type")]
    pub group_type: Type,
}

/// Information about another user
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Member {
    /// "hypothes.is"
    pub authority: String,
    /// string [ 3 .. 30 ] characters ^[A-Za-z0-9._]+$
    pub username: String,
    /// string^acct:.+$
    pub userid: String,
    /// string <= 30 characters
    #[serde(default)]
    pub display_name: Option<String>,
}
