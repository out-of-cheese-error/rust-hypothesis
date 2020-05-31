//! Bulk versions of single output API functions
//! These are asynchronous and thus faster than using a loop around the single variants
use crate::annotations::{Annotation, AnnotationMaker};
use crate::groups::{Expand, Group};
use crate::{AnnotationID, GroupID, Hypothesis};
use futures::future::try_join_all;

impl Hypothesis {
    /// Create many new annotations
    ///
    /// Posts multiple new annotation objects asynchronously to Hypothesis.
    /// Returns [`Annotation`](annotations/struct.Annotation.html)s as output.
    /// See [`AnnotationMaker`'s](annotations/struct.AnnotationMaker.html) docs for examples on what you can add to an annotation.
    ///
    /// # Example
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> color_eyre::Result<()> {
    /// # use hypothesis::Hypothesis;
    /// # use hypothesis::annotations::AnnotationMaker;
    /// #     dotenv::dotenv()?;
    /// #     let username = dotenv::var("USERNAME")?;
    /// #     let developer_key = dotenv::var("DEVELOPER_KEY")?;
    /// #     let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    /// let api = Hypothesis::new(&username, &developer_key)?;
    /// let annotation_makers = vec![
    ///     AnnotationMaker {
    ///         text: "first".to_string(),
    ///         uri: "http://example.com".to_string(),
    ///         group: group_id.to_owned(),
    ///         ..Default::default()
    ///     },
    ///     AnnotationMaker {
    ///         text: "second".to_string(),
    ///         uri: "http://example.com".to_string(),
    ///         group: group_id,   
    ///         ..Default::default()
    ///     }
    /// ];
    /// let annotations = api.create_annotations(&annotation_makers).await?;
    /// assert_eq!(&annotations[0].text, "first");
    /// assert_eq!(&annotations[1].text, "second");
    /// #    api.delete_annotations(&annotations.into_iter().map(|a| a.id).collect::<Vec<_>>()).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn create_annotations(
        &self,
        annotations: &[AnnotationMaker],
    ) -> color_eyre::Result<Vec<Annotation>> {
        let futures: Vec<_> = annotations
            .iter()
            .map(|a| self.create_annotation(a))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Update many annotations at once
    pub async fn update_annotations(
        &self,
        ids: &[AnnotationID],
        annotations: &[AnnotationMaker],
    ) -> color_eyre::Result<Vec<Annotation>> {
        let futures: Vec<_> = ids
            .iter()
            .zip(annotations.iter())
            .map(|(id, a)| self.update_annotation(id, a))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Fetch multiple annotations by ID
    pub async fn fetch_annotations(
        &self,
        ids: &[AnnotationID],
    ) -> color_eyre::Result<Vec<Annotation>> {
        let futures: Vec<_> = ids.iter().map(|id| self.fetch_annotation(id)).collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Delete multiple annotations by ID
    pub async fn delete_annotations(&self, ids: &[AnnotationID]) -> color_eyre::Result<Vec<bool>> {
        let futures: Vec<_> = ids.iter().map(|id| self.delete_annotation(id)).collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Create multiple groups
    pub async fn create_groups(
        &self,
        names: &[String],
        descriptions: &[Option<String>],
    ) -> color_eyre::Result<Vec<Group>> {
        let futures: Vec<_> = names
            .iter()
            .zip(descriptions.iter())
            .map(|(name, description)| self.create_group(name, description.as_deref()))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Fetch multiple groups by ID
    pub async fn fetch_groups(
        &self,
        ids: &[GroupID],
        expands: Vec<Vec<Expand>>,
    ) -> color_eyre::Result<Vec<Group>> {
        let futures: Vec<_> = ids
            .iter()
            .zip(expands.into_iter())
            .map(|(id, expand)| self.fetch_group(id, expand))
            .collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    /// Update multiple groups
    pub async fn update_groups(
        &self,
        ids: &[GroupID],
        names: &[Option<String>],
        descriptions: &[Option<String>],
    ) -> color_eyre::Result<Vec<Group>> {
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
}
