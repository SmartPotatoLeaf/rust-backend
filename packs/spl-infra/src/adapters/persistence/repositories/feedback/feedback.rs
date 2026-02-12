use crate::adapters::persistence::entities::diagnostics::{label, prediction};
use crate::adapters::persistence::entities::feedback::{self, status};
use crate::adapters::persistence::mappers::feedback::FeedbackMapperContext;
use sea_orm::*;
use spl_domain::entities::diagnostics::Label;
use spl_domain::entities::feedback::{Feedback, FeedbackStatus};
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::diagnostics::LabelRepository;
use spl_domain::ports::repositories::feedback::{FeedbackRepository, FeedbackStatusRepository};
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

pub struct DbFeedbackRepository {
    db: DatabaseConnection,
    status_repository: Arc<dyn FeedbackStatusRepository>,
    label_repository: Arc<dyn LabelRepository>,
}

impl DbFeedbackRepository {
    pub fn new(
        db: DatabaseConnection,
        status_repository: Arc<dyn FeedbackStatusRepository>,
        label_repository: Arc<dyn LabelRepository>,
    ) -> Self {
        Self {
            db,
            status_repository,
            label_repository,
        }
    }

    async fn find_relations(
        &self,
        status_id: i32,
        label_id: Option<i32>,
    ) -> Result<(FeedbackStatus, Option<Label>)> {
        let status_future = self.status_repository.get_by_id(status_id);

        let label_future = if let Some(lid) = label_id {
            Some(self.label_repository.get_by_id(lid))
        } else {
            None
        };

        let status_opt;
        let mut label_opt = None;

        if let Some(label) = label_future {
            (status_opt, label_opt) = tokio::try_join!(status_future, label)?
        } else {
            status_opt = status_future.await?;
        }

        let status = status_opt.ok_or_else(|| {
            AppError::NotFound(format!("Feedback status with id {} not found", status_id))
        })?;

        let label =
            if let Some(lid) = label_id {
                Some(label_opt.ok_or_else(|| {
                    AppError::NotFound(format!("Label with id {} not found", lid))
                })?)
            } else {
                None
            };

        Ok((status, label))
    }

    fn map_model(
        (model, status, label): (feedback::Model, FeedbackStatus, Option<Label>),
    ) -> Result<Feedback> {
        let context = FeedbackMapperContext {
            status,
            correct_label: label,
        };

        model.into_with_context(context)
    }

    fn map_raw_model(
        (model, status_model, label_model): (
            feedback::Model,
            Option<status::Model>,
            Option<label::Model>,
        ),
    ) -> Result<Feedback> {
        let status = status_model
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Feedback status with id {} not found",
                    model.status_id
                ))
            })?
            .into();

        DbFeedbackRepository::map_model((model, status, label_model.map(Into::into)))
    }

    async fn find(&self, select: Select<feedback::Entity>) -> Result<Vec<Feedback>> {
        select
            .find_also_related(status::Entity)
            .find_also_related(label::Entity)
            .all(&self.db)
            .await?
            .into_iter()
            .map(DbFeedbackRepository::map_raw_model)
            .collect()
    }

    fn create_prediction_select(
        select: Select<feedback::Entity>,
        user_id: Uuid,
    ) -> Select<feedback::Entity> {
        select
            .join(JoinType::LeftJoin, feedback::Relation::Prediction.def())
            .filter(prediction::Column::UserId.eq(user_id))
    }

    async fn with_map<F>(&self, action: F) -> Result<Feedback>
    where
        F: AsyncFnOnce() -> Result<(feedback::Model, FeedbackStatus, Option<Label>)>,
    {
        DbFeedbackRepository::map_model(action().await?)
    }
}

#[async_trait::async_trait]
impl CrudRepository<Feedback, Uuid> for DbFeedbackRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Feedback>> {
        let result = self
            .find(feedback::Entity::find_by_id(id))
            .await?
            .first()
            .cloned();

        Ok(result)
    }

    async fn create(&self, entity: Feedback) -> Result<Feedback> {
        self.with_map(|| async {
            let (status, correct_label) = self
                .find_relations(
                    entity.status.id,
                    entity.correct_label.as_ref().map(|l| l.id),
                )
                .await?;

            let model = crud::create_model::<feedback::Entity, Feedback>(&self.db, entity).await?;

            Ok((model, status, correct_label))
        })
        .await
    }

    async fn update(&self, entity: Feedback) -> Result<Feedback> {
        self.with_map(|| async {
            let (status, correct_label) = self
                .find_relations(
                    entity.status.id,
                    entity.correct_label.as_ref().map(|l| l.id),
                )
                .await?;

            let model = crud::update_model::<feedback::Entity, Feedback>(&self.db, entity).await?;

            Ok((model, status, correct_label))
        })
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<Feedback> {
        let target = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Feedback not found: {}", id)))?;

        crud::delete_model::<feedback::Entity, Feedback, Uuid>(&self.db, id).await?;

        Ok(target)
    }
}

#[async_trait::async_trait]
impl FeedbackRepository for DbFeedbackRepository {
    async fn get_all_by_user_id(&self, user_id: Uuid) -> Result<Vec<Feedback>> {
        // Then get all feedbacks for those predictions
        self.find(DbFeedbackRepository::create_prediction_select(
            feedback::Entity::find(),
            user_id,
        ))
        .await
    }

    async fn get_by_id_and_user_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Feedback>> {
        let result = self
            .find(DbFeedbackRepository::create_prediction_select(
                feedback::Entity::find_by_id(id),
                user_id,
            ))
            .await?
            .first()
            .cloned();

        Ok(result)
    }

    async fn delete_by_id_and_user_id(&self, id: Uuid, user_id: Uuid) -> Result<Feedback> {
        let result = self
            .get_by_id_and_user_id(id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Feedback not found: {}", id)))?;

        crud::delete_model::<feedback::Entity, Feedback, Uuid>(&self.db, id).await?;

        Ok(result)
    }

    async fn get_by_prediction_id(&self, prediction_id: Uuid) -> Result<Option<Feedback>> {
        let result = self
            .find(feedback::Entity::find().filter(feedback::Column::PredictionId.eq(prediction_id)))
            .await?
            .first()
            .cloned();

        Ok(result)
    }

    async fn get_by_user_and_prediction_id(
        &self,
        user_id: Uuid,
        prediction_id: Uuid,
    ) -> Result<Option<Feedback>> {
        let result = self
            .find(DbFeedbackRepository::create_prediction_select(
                feedback::Entity::find().filter(feedback::Column::PredictionId.eq(prediction_id)),
                user_id,
            ))
            .await?
            .first()
            .cloned();

        Ok(result)
    }
}
