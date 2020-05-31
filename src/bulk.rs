//! Bulk versions of single input API functions
use crate::annotations::{Annotation, AnnotationMaker};
use crate::groups::{Expand, Group};
use crate::{AnnotationID, GroupID, Hypothesis};
use futures::future::try_join_all;

impl Hypothesis {
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

    pub async fn fetch_annotations(
        &self,
        ids: &[AnnotationID],
    ) -> color_eyre::Result<Vec<Annotation>> {
        let futures: Vec<_> = ids.iter().map(|id| self.fetch_annotation(id)).collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

    pub async fn delete_annotations(&self, ids: &[AnnotationID]) -> color_eyre::Result<Vec<bool>> {
        let futures: Vec<_> = ids.iter().map(|id| self.delete_annotation(id)).collect();
        Ok(async { try_join_all(futures).await }.await?)
    }

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
