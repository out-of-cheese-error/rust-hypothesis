//! Objects related to the "annotations" endpoint

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
#[cfg(feature = "cli")]
use structopt::StructOpt;

use crate::{is_default, UserAccountID};

#[cfg_attr(feature = "cli", derive(StructOpt))]
#[cfg_attr(
    feature = "cli",
    structopt(
        about = "Create or update an annotation",
        long_about = "Create / update and upload an annotation to your Hypothesis"
    )
)]
/// Struct to create and update annotations
///
/// For creating a new annotation, all fields except uri are optional, i.e. leave as default.
/// For updating an existing annotation, all fields are optional.
///
/// # Example
/// ```
/// use hypothesis::annotations::{InputAnnotationBuilder, TargetBuilder, Selector};
/// # #[tokio::main]
/// # async fn main() -> color_eyre::Result<()> {
/// // A simple annotation
/// let annotation_simple = InputAnnotationBuilder::default()
///     .uri("https://www.example.com")
///     .text("My new annotation").build()?;
///
/// // A complex annotation
/// let annotation_complex = InputAnnotationBuilder::default()
///     .uri("https://www.example.com")
///     .text("this is a comment")
///     .target(TargetBuilder::default().source("https://www.example.com")
///         .selector(vec![Selector::new_quote("exact text in website to highlight",
///                                             "prefix of text",
///                                             "suffix of text")]).build()?)
///     .tags(vec!["tag1".into(), "tag2".into()])
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Serialize, Debug, Default, Clone, Builder, PartialEq)]
#[builder(default, build_fn(name = "builder"))]
pub struct InputAnnotation {
    /// URI that this annotation is attached to.
    ///
    /// Can be a URL (a web page address) or a URN representing another kind of resource such as
    /// DOI (Digital Object Identifier) or a PDF fingerprint.
    #[serde(skip_serializing_if = "is_default")]
    #[builder(setter(into))]
    pub uri: String,
    /// Annotation text / comment given by user
    ///
    /// This is NOT the selected text on the web-page
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub text: String,
    /// Tags attached to the annotation
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(long))]
    #[builder(setter(strip_option), default)]
    pub tags: Option<Vec<String>>,
    /// Further metadata about the target document
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(skip))]
    #[builder(setter(strip_option), default)]
    pub document: Option<Document>,
    #[serde(skip_serializing_if = "is_default")]
    /// The unique identifier for the annotation's group.
    ///
    /// If an annotation is a reply to another
    /// annotation (see `references`), this field will be ignored —
    /// replies belong to the same group as their parent annotations.
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub group: String,
    /// Which part of the document does the annotation target?
    ///
    /// If left as default then the annotation is linked to the whole page.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(skip))]
    pub target: Target,
    /// Annotation IDs for any annotations this annotation references (e.g. is a reply to)
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(long))]
    pub references: Vec<String>,
}

impl InputAnnotationBuilder {
    /// Builds a new `InputAnnotation`.
    pub fn build(&self) -> color_eyre::Result<InputAnnotation> {
        self.builder().map_err(|e| eyre!(e))
    }
}

#[derive(Serialize, Debug, Default, Clone, PartialEq, Builder)]
#[builder(build_fn(name = "builder"))]
pub struct Document {
    #[serde(skip_serializing_if = "is_default")]
    pub title: Vec<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[builder(setter(strip_option), default)]
    pub dc: Option<Dc>,
    #[serde(skip_serializing_if = "is_default")]
    #[builder(setter(strip_option), default)]
    pub highwire: Option<HighWire>,
    #[serde(skip_serializing_if = "is_default")]
    pub link: Vec<Link>,
}

impl DocumentBuilder {
    /// Builds a new `Document`.
    pub fn build(&self) -> color_eyre::Result<Document> {
        self.builder().map_err(|e| eyre!(e))
    }
}

#[derive(Serialize, Default, Debug, Clone, PartialEq)]
pub struct HighWire {
    #[serde(skip_serializing_if = "is_default")]
    pub doi: Vec<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub pdf_url: Vec<String>,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "is_default", rename = "type")]
    pub link_type: String,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct Dc {
    #[serde(skip_serializing_if = "is_default")]
    pub identifier: Vec<String>,
}

/// Full representation of an Annotation resource and applicable relationships.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Annotation {
    /// Annotation ID
    pub id: String,
    /// Date of creation
    pub created: DateTime<Utc>,
    /// Date of last update
    pub updated: DateTime<Utc>,
    /// User account ID in the format "acct:<username>@<authority>"
    pub user: UserAccountID,
    /// URL of document this annotation is attached to
    pub uri: String,
    /// The text content of the annotation body (NOT the selected text in the document)
    pub text: String,
    /// Tags attached to annotation
    pub tags: Vec<String>,
    /// The unique identifier for the annotation's group
    pub group: String,
    pub permissions: Permissions,
    /// Which part of the document does the annotation target.
    pub target: Vec<Target>,
    /// An object containing hypermedia links for this annotation
    pub links: HashMap<String, String>,
    /// Whether this annotation is hidden from public view
    pub hidden: bool,
    /// Whether this annotation has one or more flags for moderation
    pub flagged: bool,
    /// Annotation IDs for any annotations this annotation references (e.g. is a reply to)
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub user_info: Option<UserInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    /// The annotation creator's display name
    #[serde(default)]
    pub display_name: String,
}

/// > While the API accepts arbitrary Annotation selectors in the target.selector property,
/// > the Hypothesis client currently supports TextQuoteSelector, RangeSelector and TextPositionSelector selector.
/// [Hypothesis API v1.0.0](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/annotations/paths/~1annotations/post)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Builder)]
#[builder(build_fn(name = "builder"))]
pub struct Target {
    /// The target URI for the annotation
    /// Leave empty when creating an annotation
    #[serde(skip_serializing_if = "is_default")]
    #[builder(setter(into))]
    pub source: String,
    /// An array of selectors that refine this annotation's target
    #[serde(default, skip_serializing_if = "is_default")]
    pub selector: Vec<Selector>,
}

impl TargetBuilder {
    /// Builds a new `Target`.
    pub fn build(&self) -> color_eyre::Result<Target> {
        self.builder().map_err(|e| eyre!(e))
    }
}

/// > Many Annotations refer to part of a resource, rather than all of it, as the Target.
/// > We call that part of the resource a Segment (of Interest). A Selector is used to describe how
/// > to determine the Segment from within the Source resource.
/// [Web Annotation Data Model - Selectors](https://www.w3.org/TR/annotation-model/#selectors)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Selector {
    TextQuoteSelector(TextQuoteSelector),
    /// > Selections made by users may be extensive and/or cross over internal boundaries in the
    /// > representation, making it difficult to construct a single selector that robustly describes
    /// > the correct content. A Range Selector can be used to identify the beginning and the end of
    /// > the selection by using other Selectors. In this way, two points can be accurately identified
    /// > using the most appropriate selection mechanisms, and then linked together to form the selection.
    /// > The selection consists of everything from the beginning of the starting selector through to the
    /// > beginning of the ending selector, but not including it.
    /// [Web Annotation Data Model - Range Selector](https://www.w3.org/TR/annotation-model/#range-selector)
    /// NOTE - the Hypothesis API doesn't seem to follow this standard for RangeSelector so this just returns a HashMap for now
    /// TODO: make RangeSelector a struct
    RangeSelector(HashMap<String, serde_json::Value>),
    TextPositionSelector(TextPositionSelector),
}

impl Selector {
    pub fn new_quote(exact: &str, prefix: &str, suffix: &str) -> Self {
        Self::TextQuoteSelector(TextQuoteSelector {
            exact: exact.to_string(),
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
        })
    }
}

/// > This Selector describes a range of text by copying it, and including some of the text
/// > immediately before (a prefix) and after (a suffix) it to distinguish between multiple
/// > copies of the same sequence of characters.
///
/// > For example, if the document were again "abcdefghijklmnopqrstuvwxyz", one could select
/// > "efg" by a prefix of "abcd", the match of "efg" and a suffix of "hijk".
/// [Web Annotation Data Model - Text Quote Selector](https://www.w3.org/TR/annotation-model/#text-quote-selector)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextQuoteSelector {
    /// A copy of the text which is being selected, after normalization.
    pub exact: String,
    /// A snippet of text that occurs immediately before the text which is being selected.
    pub prefix: String,
    /// The snippet of text that occurs immediately after the text which is being selected.
    pub suffix: String,
}

/// >  This Selector describes a range of text by recording the start and end positions of the
/// > selection in the stream. Position 0 would be immediately before the first character, position
/// > 1 would be immediately before the second character, and so on. The start character is thus
/// > included in the list, but the end character is not.
///
/// > For example, if the document was "abcdefghijklmnopqrstuvwxyz", the start was 4, and the end
/// > was 7, then the selection would be "efg".
/// [Web Annotation Data Model - Text Position Selector](https://www.w3.org/TR/annotation-model/#text-position-selector)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextPositionSelector {
    /// The starting position of the segment of text. The first character in the full text is
    /// character position 0, and the character is included within the segment.
    pub start: u64,
    /// The end position of the segment of text. The character is not included within the segment.
    pub end: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    Created,
    Updated,
    Id,
    Group,
    User,
}

impl Default for Sort {
    fn default() -> Self {
        Self::Updated
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Self::Desc
    }
}

/// Options to filter and sort search results. See [the Hypothesis API docs](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/annotations/paths/~1search/get) for more details on using these fields
#[cfg_attr(feature = "cli", derive(StructOpt))]
#[derive(Serialize, Debug, Clone, PartialEq, Builder, Default)]
#[builder(build_fn(name = "builder"), default)]
pub struct SearchQuery {
    /// The maximum number of annotations to return.
    ///
    /// Default: 20. Range: [ 0 .. 200 ]
    #[builder(default = "20")]
    #[cfg_attr(feature = "cli", structopt(default_value = "20", long))]
    pub limit: u8,
    /// The field by which annotations should be sorted
    /// One of created, updated, id, group, user
    ///
    /// Default: updated
    #[cfg_attr(feature = "cli", structopt(default_value = "updated", long, possible_values = & Sort::variants()))]
    pub sort: Sort,
    /// Example: "2019-01-03T19:46:09.334Z"
    ///
    /// Define a start point for a subset (page) of annotation search results.
    /// NOTE: make sure to set sort to `Sort::Asc` if using `search_after`
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub search_after: String,
    /// The number of initial annotations to skip in the result set.
    ///
    /// Default: 0. Range: <= 9800.
    /// search_after is more efficient.
    #[cfg_attr(feature = "cli", structopt(default_value = "0", long))]
    pub offset: usize,
    /// The order in which the results should be sorted.
    /// One of asc, desc
    ///
    /// Default: desc
    #[cfg_attr(feature = "cli", structopt(default_value = "desc", long, possible_values = & Order::variants()))]
    pub order: Order,
    /// Limit the results to annotations matching the specific URI or equivalent URIs.
    ///
    /// URI can be a URL (a web page address) or a URN representing another kind of resource such
    /// as DOI (Digital Object Identifier) or a PDF fingerprint.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub uri: String,
    /// Limit the results to annotations containing the given keyword (tokenized chunk) in the URI.
    /// The value must exactly match an individual URI keyword.
    ///
    #[serde(rename = "uri.parts", skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub uri_parts: String,
    /// Limit the results to annotations whose URIs match the wildcard pattern.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub wildcard_uri: String,
    /// Limit the results to annotations made by the specified user. (in the format `acct:<username>@<authority>`)
    #[serde(skip_serializing_if = "is_default", serialize_with = "serialize_user")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub user: UserAccountID,
    /// Limit the results to annotations made in the specified group (by group ID).
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub group: String,
    /// Limit the results to annotations tagged with the specified value.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub tag: String,
    /// Similar to tag but allows a list of multiple tags.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(long))]
    pub tags: Vec<String>,
    /// Limit the results to annotations who contain the indicated keyword in any of the following fields:
    /// `quote`, `tags`, `text`, `url`
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub any: String,
    /// Limit the results to annotations that contain this text inside the text that was annotated.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub quote: String,
    /// Returns annotations that are replies to this parent annotation ID.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub references: String,
    /// Limit the results to annotations that contain this text in their textual body.
    #[serde(skip_serializing_if = "is_default")]
    #[cfg_attr(feature = "cli", structopt(default_value, long))]
    #[builder(setter(into))]
    pub text: String,
}

impl SearchQueryBuilder {
    /// Builds a new `SearchQuery`.
    pub fn build(&self) -> color_eyre::Result<SearchQuery> {
        self.builder().map_err(|e| eyre!(e))
    }
}

fn serialize_user<S>(x: &UserAccountID, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.0)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Permissions {
    pub read: Vec<String>,
    pub delete: Vec<String>,
    pub admin: Vec<String>,
    pub update: Vec<String>,
}
