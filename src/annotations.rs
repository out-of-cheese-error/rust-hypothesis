use crate::{is_default, APIError, AnnotationID, GroupID, Hypothesis, UserAccountID, API_URL};

use chrono::{DateTime, Utc};
use color_eyre::Help;
use eyre::WrapErr;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Hypothesis {
    /// Create a new annotation
    ///
    /// Posts a new annotation object to Hypothesis.
    /// Returns an [`Annotation`](annotations/struct.Annotation.html) as output.
    /// See [`AnnotationMaker`'s](annotations/struct.AnnotationMaker.html) docs for examples on what you can add to an annotation.
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    ///     use hypothesis::annotations::AnnotationMaker;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let annotation = api.create_annotation(&AnnotationMaker {
    ///                 text: "string",
    ///                 uri: "http://example.com",
    ///                 group: group_id,
    ///                 ..Default::default()
    ///             })?;
    ///     assert_eq!(&annotation.text, "string");
    /// #    api.delete_annotation(&annotation.id)?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn create_annotation(
        &self,
        annotation: &AnnotationMaker,
    ) -> color_eyre::Result<Annotation> {
        let text = self
            .client
            .post(&format!("{}/annotations", API_URL))
            .json(annotation)
            .send()?
            .text()?;
        let result = serde_json::from_str::<Annotation>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("Make sure the AnnotationMaker is valid");
        Ok(result?)
    }

    /// Update an existing annotation
    ///
    /// Change any field in an existing annotation. Returns the modified [`Annotation`](annotations/struct.Annotation.html)
    /// Fields in  [`AnnotationMaker`](annotations/struct.AnnotationMaker.html) which are left as default are not modified in the annotation
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    ///     use hypothesis::annotations::AnnotationMaker;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    /// #    let annotation = api.create_annotation(&AnnotationMaker {
    /// #                 text: "string",
    /// #                 uri: "http://example.com",
    /// #                 group: group_id,
    /// #                 ..Default::default()
    /// #             })?;
    /// #    let annotation_id = annotation.id.to_owned();    
    ///     let updated_annotation = api.update_annotation(&annotation_id, &AnnotationMaker {
    ///             tags: vec!["tag1", "tag2"],
    ///             text: "New String",
    ///             ..Default::default()
    ///         })?;
    ///     assert_eq!(updated_annotation.id, annotation_id);
    ///     assert_eq!(&updated_annotation.text, "New String");
    /// #    api.delete_annotation(&updated_annotation.id)?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn update_annotation(
        &self,
        id: &AnnotationID,
        annotation: &AnnotationMaker,
    ) -> color_eyre::Result<Annotation> {
        let text = self
            .client
            .patch(&format!("{}/annotations/{}", API_URL, id))
            .json(&annotation)
            .send()?
            .text()?;
        let result = serde_json::from_str::<Annotation>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("Make sure the AnnotationMaker is valid");
        Ok(result?)
    }

    /// Search for annotations with optional filters
    ///
    /// Returns a list of annotations matching the search query.
    /// See  [`SearchQuery`](annotations/struct.SearchQuery.html) for more filtering options
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    ///     use hypothesis::annotations::SearchQuery;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///     /// Search for your own annotations:
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    ///     let search_query = SearchQuery {
    ///             limit: 30,
    ///             user: &api.user,
    ///             ..Default::default()
    ///         };
    ///      let search_results = api.search_annotations(&search_query)?;
    /// #     assert!(!search_results.is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    pub fn search_annotations(&self, query: &SearchQuery) -> color_eyre::Result<Vec<Annotation>> {
        let query: HashMap<String, serde_json::Value> =
            serde_json::from_str(&serde_json::to_string(&query)?)?;
        let url = Url::parse_with_params(
            &format!("{}/search", API_URL),
            &query
                .into_iter()
                .map(|(k, v)| (k, v.to_string().replace('"', "")))
                .collect::<Vec<_>>(),
        )?;
        let text = self.client.get(url).send()?.text()?;
        let result = serde_json::from_str::<SearchResult>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("Make sure the SearchQuery is valid");
        Ok(result?.rows)
    }

    /// Fetch annotation by ID
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #    use hypothesis::annotations::AnnotationMaker;
    /// #    dotenv::dotenv()?;
    /// #    let username = dotenv::var("USERNAME")?;
    /// #    let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    /// #    let annotation = api.create_annotation(&AnnotationMaker {
    /// #                 text: "string",
    /// #                 uri: "http://example.com",
    /// #                 group: group_id,
    /// #                 ..Default::default()
    /// #             })?;
    /// #    let annotation_id = annotation.id.to_owned();    
    ///     let annotation = api.fetch_annotation(&annotation_id)?;
    ///     assert_eq!(annotation.id, annotation_id);
    /// #    api.delete_annotation(&annotation.id)?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn fetch_annotation(&self, id: &AnnotationID) -> color_eyre::Result<Annotation> {
        let text = self
            .client
            .get(&format!("{}/annotations/{}", API_URL, id))
            .send()?
            .text()?;
        let result = serde_json::from_str::<Annotation>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("Make sure the given AnnotationID exists");
        Ok(result?)
    }

    /// Delete annotation by ID
    ///
    /// # Example
    /// ```
    /// # fn main() -> color_eyre::Result<()> {
    ///     use hypothesis::Hypothesis;
    /// #    use hypothesis::annotations::AnnotationMaker;
    /// #    dotenv::dotenv()?;
    /// #    let username = dotenv::var("USERNAME")?;
    /// #    let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    ///     let api = Hypothesis::new(&username, &developer_key)?;
    /// #    let annotation = api.create_annotation(&AnnotationMaker {
    /// #                 text: "string",
    /// #                 uri: "http://example.com",
    /// #                 group: group_id,
    /// #                 ..Default::default()
    /// #             })?;
    /// #    let annotation_id = annotation.id.to_owned();    
    ///     let deleted = api.delete_annotation(&annotation_id)?;
    ///     assert!(deleted);
    ///     assert!(api.fetch_annotation(&annotation_id).is_err());
    /// #    Ok(())
    /// # }
    /// ```
    pub fn delete_annotation(&self, id: &AnnotationID) -> color_eyre::Result<bool> {
        let text = self
            .client
            .delete(&format!("{}/annotations/{}", API_URL, id))
            .send()?
            .text()?;
        let result = serde_json::from_str::<DeletionResult>(&text)
            .wrap_err(serde_json::from_str::<APIError>(&text).unwrap_or_default())
            .suggestion("Make sure the given AnnotationID exists");
        Ok(result?.deleted)
    }

    /// Flag an annotation
    ///
    /// Flag an annotation for review (moderation). The moderator of the group containing the
    /// annotation will be notified of the flag and can decide whether or not to hide the
    /// annotation. Note that flags persist and cannot be removed once they are set.
    pub fn flag_annotation(&self, id: &AnnotationID) -> color_eyre::Result<()> {
        let text = self
            .client
            .put(&format!("{}/annotations/{}/flag", API_URL, id))
            .send()?
            .text()?;
        let error = serde_json::from_str::<APIError>(&text);
        if let Ok(error) = error {
            Err(error).suggestion("Make sure the given AnnotationID exists")
        } else {
            Ok(())
        }
    }

    /// Hide an annotation
    ///
    /// Hide an annotation. The authenticated user needs to have the moderate permission for the
    /// group that contains the annotation — this permission is granted to the user who created the group.
    pub fn hide_annotation(&self, id: &AnnotationID) -> color_eyre::Result<()> {
        let text = self
            .client
            .put(&format!("{}/annotations/{}/hide", API_URL, id))
            .send()?
            .text()?;
        let error = serde_json::from_str::<APIError>(&text);
        if let Ok(error) = error {
            Err(error).suggestion("Make sure the given AnnotationID exists")
        } else {
            Ok(())
        }
    }

    /// Show an annotation
    ///
    /// Show/"un-hide" an annotation. The authenticated user needs to have the moderate permission
    /// for the group that contains the annotation—this permission is granted to the user who created the group.
    pub fn show_annotation(&self, id: &AnnotationID) -> color_eyre::Result<()> {
        let text = self
            .client
            .delete(&format!("{}/annotations/{}/hide", API_URL, id))
            .send()?
            .text()?;
        let error = serde_json::from_str::<APIError>(&text);
        if let Ok(error) = error {
            Err(error).suggestion("Make sure the given AnnotationID exists")
        } else {
            Ok(())
        }
    }
}

/// Struct to create and update annotations
///
/// For creating a new annotation, all fields except uri are optional, i.e. leave as default.
/// For updating an existing annotation, all fields are optional.
///
/// # Example
/// ```
/// # use hypothesis::annotations::AnnotationMaker;
/// /// A simple annotation
/// let annotation_simple = AnnotationMaker {
///     uri: "https://example.com",
///     text: "My new annotation",
///     .. Default::default()
/// };
///
/// /// A complex annotation
/// let annotation_complex = AnnotationMaker {
///     uri: "https://wikipedia.com",
///     
///     .. Default::default()
/// };
/// ```
#[derive(Serialize, Debug, Default, Clone)]
pub struct AnnotationMaker<'a> {
    /// URL to which this annotation is attached
    #[serde(skip_serializing_if = "is_default")]
    pub uri: &'a str,
    /// Annotation text / comment given by user
    /// This is NOT the selected text on the web-page
    #[serde(skip_serializing_if = "is_default")]
    pub text: &'a str,
    /// Tags attached to the annotation
    #[serde(skip_serializing_if = "is_default")]
    pub tags: Vec<&'a str>,
    /// Further metadata about the target document
    #[serde(skip_serializing_if = "is_default")]
    pub document: Option<Document<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    /// The unique identifier for the annotation's group. If an annotation is a reply to another
    /// annotation (see `references`), this field will be ignored — replies belong to the same group
    /// as their parent annotations.
    pub group: GroupID,
    /// Which part of the document does the annotation target?
    /// If left as default then the annotation is linked to the whole page.
    #[serde(skip_serializing_if = "is_default")]
    pub target: Target,
    /// Annotation IDs for any annotations this annotation references (e.g. is a reply to)
    #[serde(skip_serializing_if = "is_default")]
    pub references: Vec<AnnotationID>,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct Document<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub title: Vec<&'a str>,
    #[serde(skip_serializing_if = "is_default")]
    pub dc: Option<Dc<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    pub highwire: Option<HighWire<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    pub link: Vec<Link<'a>>,
}

#[derive(Serialize, Default, Debug, Clone, PartialEq)]
pub struct HighWire<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub doi: Vec<&'a str>,
    #[serde(skip_serializing_if = "is_default")]
    pub pdf_url: Vec<&'a str>,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct Link<'a> {
    pub href: &'a str,
    #[serde(skip_serializing_if = "is_default", rename = "type")]
    pub link_type: &'a str,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct Dc<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub identifier: Vec<&'a str>,
}

/// Full representation of Annotation resource and applicable relationships.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Annotation {
    /// Annotation ID
    pub id: AnnotationID,
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
    pub group: GroupID,
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
    pub references: Vec<AnnotationID>,
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
/// - [Hypothesis API v1.0.0](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/annotations/paths/~1annotations/post)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Target {
    /// The target URI for the annotation
    /// Leave empty when creating an annotation
    #[serde(skip_serializing_if = "is_default")]
    pub source: String,
    /// An array of selectors that refine this annotation's target
    #[serde(default, skip_serializing_if = "is_default")]
    pub selector: Vec<Selector>,
}

/// > Many Annotations refer to part of a resource, rather than all of it, as the Target.
/// > We call that part of the resource a Segment (of Interest). A Selector is used to describe how
/// > to determine the Segment from within the Source resource.
/// - [Web Annotation Data Model - Selectors](https://www.w3.org/TR/annotation-model/#selectors)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Selector {
    TextQuoteSelector(TextQuoteSelector),
    // > Selections made by users may be extensive and/or cross over internal boundaries in the
    /// > representation, making it difficult to construct a single selector that robustly describes
    /// > the correct content. A Range Selector can be used to identify the beginning and the end of
    /// > the selection by using other Selectors. In this way, two points can be accurately identified
    /// > using the most appropriate selection mechanisms, and then linked together to form the selection.
    /// > The selection consists of everything from the beginning of the starting selector through to the
    /// > beginning of the ending selector, but not including it.
    /// - [Web Annotation Data Model - Range Selector](https://www.w3.org/TR/annotation-model/#range-selector)
    /// NOTE - the Hypothesis API doesn't seem to follow this standard for RangeSelector so this just returns a HashMap for now
    /// TODO: make RangeSelector a struct
    RangeSelector(HashMap<String, serde_json::Value>),
    TextPositionSelector(TextPositionSelector),
}

/// > This Selector describes a range of text by copying it, and including some of the text
/// > immediately before (a prefix) and after (a suffix) it to distinguish between multiple
/// > copies of the same sequence of characters.
///
/// > For example, if the document were again "abcdefghijklmnopqrstuvwxyz", one could select
/// > "efg" by a prefix of "abcd", the match of "efg" and a suffix of "hijk".
/// - [Web Annotation Data Model - Text Quote Selector](https://www.w3.org/TR/annotation-model/#text-quote-selector)
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
/// - [Web Annotation Data Model - Text Position Selector](https://www.w3.org/TR/annotation-model/#text-position-selector)
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

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SearchQuery<'a> {
    /// The maximum number of annotations to return. Default: 20. Range: [ 0 .. 200 ]
    pub limit: u8,
    /// The field by which annotations should be sorted. Default: Updated
    pub sort: Sort,
    /// Example: "2019-01-03T19:46:09.334Z"
    ///
    /// Define a start point for a subset (page) of annotation search results.
    ///
    /// Note: search_after provides an efficient, stateless paging mechanism. Its use is preferred over that of offset.
    ///
    /// See [the Hypothesis API docs](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/annotations/paths/~1search/get) for more details on using this field
    #[serde(skip_serializing_if = "is_default")]
    pub search_after: &'a str,
    /// The number of initial annotations to skip in the result set. Default: 0. Range: <= 9800
    /// May be used for pagination of result sets. The usage of search_after is preferred, especially for large batches, as it is considerably more efficient.
    pub offset: usize,
    /// The order in which the results should be sorted. Default: Desc
    pub order: Order,
    /// Limit the results to annotations matching the specific URI or equivalent URIs.
    ///
    /// URI can be a URL (a web page address) or a URN representing another kind of resource such as DOI (Digital Object Identifier) or a PDF fingerprint.
    #[serde(skip_serializing_if = "is_default")]
    pub uri: &'a str,
    /// Limit the results to annotations containing the given keyword (tokenized chunk) in the URI. The value must exactly match an individual URI keyword.
    /// See [the Hypothesis API docs](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/annotations/paths/~1search/get) for more details on using this field
    #[serde(rename = "uri.parts", skip_serializing_if = "is_default")]
    pub uri_parts: &'a str,
    /// Limit the results to annotations whose URIs match the wildcard pattern.
    /// See [the Hypothesis API docs](https://h.readthedocs.io/en/latest/api-reference/v1/#tag/annotations/paths/~1search/get) for more details on using this field
    #[serde(skip_serializing_if = "is_default")]
    pub wildcard_uri: &'a str,
    /// Limit the results to annotations made by the specified user. (in the format "acct:<username>@<authority>")
    #[serde(skip_serializing_if = "is_default")]
    pub user: &'a str,
    /// Limit the results to annotations made in the specified group (by group ID).
    #[serde(skip_serializing_if = "is_default")]
    pub group: GroupID,
    /// Limit the results to annotations tagged with the specified value.
    #[serde(skip_serializing_if = "is_default")]
    pub tag: &'a str,
    /// Similar to tag but allows a comma-separated list of multiple tags.
    #[serde(skip_serializing_if = "is_default")]
    pub tags: Vec<&'a str>,
    /// Limit the results to annotations who contain the indicated keyword in any of the following fields:
    ///
    /// * `quote`
    /// * `tags`
    /// * `text`
    /// * `url`
    #[serde(skip_serializing_if = "is_default")]
    pub any: &'a str,
    /// Limit the results to annotations that contain this text inside the text that was annotated.
    #[serde(skip_serializing_if = "is_default")]
    pub quote: &'a str,
    /// Returns annotations that are replies to this parent annotation ID.
    #[serde(skip_serializing_if = "is_default")]
    pub references: AnnotationID,
    /// Limit the results to annotations that contain this text in their textual body.
    #[serde(skip_serializing_if = "is_default")]
    pub text: &'a str,
}

impl<'a> Default for SearchQuery<'a> {
    fn default() -> Self {
        SearchQuery {
            limit: 20,
            sort: Default::default(),
            search_after: "",
            offset: 0,
            order: Default::default(),
            uri: "",
            uri_parts: "",
            wildcard_uri: "",
            user: "",
            group: "".into(),
            tag: "",
            tags: vec![],
            any: "",
            quote: "",
            references: "".into(),
            text: "",
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct SearchResult {
    rows: Vec<Annotation>,
    total: usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct DeletionResult {
    id: AnnotationID,
    deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Permissions {
    pub read: Vec<String>,
    pub delete: Vec<String>,
    pub admin: Vec<String>,
    pub update: Vec<String>,
}
