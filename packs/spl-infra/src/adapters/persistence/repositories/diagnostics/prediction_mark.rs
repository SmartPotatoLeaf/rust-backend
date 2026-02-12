use crate::adapters::persistence::entities::diagnostics::{mark_type, prediction_mark};
use crate::adapters::persistence::mappers::diagnostics::prediction_mark::PredictionMarkMapperContext;
use sea_orm::*;
use spl_domain::entities::diagnostics::{MarkType, PredictionMark};
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::diagnostics::{MarkTypeRepository, PredictionMarkRepository};
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

pub struct DbPredictionMarkRepository {
    db: DatabaseConnection,
    mark_type_repository: Arc<dyn MarkTypeRepository>,
}

impl DbPredictionMarkRepository {
    pub fn new(db: DatabaseConnection, mark_type_repository: Arc<dyn MarkTypeRepository>) -> Self {
        Self {
            db,
            mark_type_repository,
        }
    }

    async fn find_relations(&self, mark_type_id: i32) -> Result<MarkType> {
        self.mark_type_repository
            .get_by_id(mark_type_id)
            .await?
            .ok_or(AppError::NotFound(format!(
                "No mark type with id {}",
                mark_type_id
            )))
    }

    async fn validate_relations(&self, mark: &PredictionMark) -> Result<MarkType> {
        self.find_relations(mark.mark_type.id).await
    }

    async fn find_and_validate_mark_types(&self, marks: &[PredictionMark]) -> Result<()> {
        let unique_type_ids: Vec<i32> = marks
            .iter()
            .map(|m| m.mark_type.id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        if unique_type_ids.is_empty() {
            return Err(AppError::NotFound(
                "No mark types found in request".into(),
            ));
        }

        let found_types = self
            .mark_type_repository
            .get_by_ids(unique_type_ids.clone())
            .await?;

        if found_types.len() != unique_type_ids.len() {
            return Err(AppError::NotFound(
                "One or more MarkTypes not found".into(),
            ));
        }

        Ok(())
    }

    fn map_model(
        models: (prediction_mark::Model, Option<mark_type::Model>),
    ) -> Result<PredictionMark> {
        let (mark_model, type_model) = models;
        let type_model = type_model.ok_or_else(|| {
            AppError::NotFound(format!("MarkType not found for Mark {}", mark_model.id))
        })?;

        let mark_type: MarkType = type_model.into();
        let context = PredictionMarkMapperContext { mark_type };
        mark_model.into_with_context(context)
    }

    async fn with_map<F>(&self, action: F) -> Result<PredictionMark>
    where
        F: AsyncFnOnce() -> Result<(prediction_mark::Model, MarkType)>,
    {
        let (model, mark_type) = action().await?;
        let context = PredictionMarkMapperContext { mark_type };
        model.into_with_context(context)
    }

    async fn find(
        &self,
        condition: Condition,
    ) -> Result<Vec<(prediction_mark::Model, Option<mark_type::Model>)>> {
        prediction_mark::Entity::find()
            .filter(condition)
            .find_also_related(mark_type::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from)
    }
}

#[async_trait::async_trait]
impl CrudRepository<PredictionMark, Uuid> for DbPredictionMarkRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<PredictionMark>> {
        let model = prediction_mark::Entity::find_by_id(id)
            .find_also_related(mark_type::Entity)
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        match model {
            Some(e) => Ok(Some(DbPredictionMarkRepository::map_model(e)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, entity: PredictionMark) -> Result<PredictionMark> {
        self.with_map(|| async {
            let mark_type = self.validate_relations(&entity).await?;
            let model = crud::create_model::<prediction_mark::Entity, PredictionMark>(
                &self.db,
                entity.clone(),
            )
            .await?;

            Ok((model, mark_type))
        })
        .await
    }

    async fn update(&self, entity: PredictionMark) -> Result<PredictionMark> {
        self.with_map(|| async {
            let mark_type = self.validate_relations(&entity).await?;
            let model = crud::update_model::<prediction_mark::Entity, PredictionMark>(
                &self.db,
                entity.clone(),
            )
            .await?;
            Ok((model, mark_type))
        })
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<PredictionMark> {
        self.with_map(|| async {
            let model =
                crud::delete_model::<prediction_mark::Entity, PredictionMark, Uuid>(&self.db, id)
                    .await?;
            let mark_type = self.find_relations(model.mark_type_id).await?;
            Ok((model, mark_type))
        })
        .await
    }
}

#[async_trait::async_trait]
impl PredictionMarkRepository for DbPredictionMarkRepository {
    async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<PredictionMark>> {
        let models = self
            .find(Condition::all().add(prediction_mark::Column::Id.is_in(ids)))
            .await?;

        models
            .into_iter()
            .map(DbPredictionMarkRepository::map_model)
            .collect()
    }
    async fn create_many(&self, marks: Vec<PredictionMark>) -> Result<Vec<PredictionMark>> {
        if marks.is_empty() {
            return Ok(vec![]);
        }

        // Validate all mark types before attempting to insert any marks
        self.find_and_validate_mark_types(&marks).await?;

        let models: Vec<prediction_mark::ActiveModel> = marks.into_iter().map(Into::into).collect();

        let results = prediction_mark::Entity::insert_many(models.clone())
            .exec_with_returning_many(&self.db)
            .await
            .map_err(AppError::from)?;

        self.get_by_ids(results.into_iter().map(|e| e.id).collect())
            .await
    }

    async fn get_by_prediction_id(&self, prediction_id: Uuid) -> Result<Vec<PredictionMark>> {
        let models = prediction_mark::Entity::find()
            .filter(prediction_mark::Column::PredictionId.eq(prediction_id))
            .find_also_related(mark_type::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        models
            .into_iter()
            .map(DbPredictionMarkRepository::map_model)
            .collect()
    }

    async fn get_by_predictions_ids(
        &self,
        prediction_ids: Vec<Uuid>,
    ) -> Result<Vec<PredictionMark>> {
        let marks = self
            .find(Condition::all().add(prediction_mark::Column::PredictionId.is_in(prediction_ids)))
            .await?;

        marks
            .into_iter()
            .map(DbPredictionMarkRepository::map_model)
            .collect()
    }
}
