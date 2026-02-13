use crate::adapters::persistence::entities::diagnostics::{label, prediction};
use crate::adapters::persistence::entities::image as image_persistence;
use crate::adapters::persistence::mappers::diagnostics::prediction::PredictionMapperContext;
use sea_orm::prelude::Expr;
use sea_orm::*;
use spl_domain::entities::diagnostics::{Label, Prediction, PredictionMark};
use spl_domain::entities::feedback::Feedback;
use spl_domain::entities::image::Image;
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::diagnostics::{
    LabelRepository, PredictionMarkRepository, PredictionRepository,
};
use spl_domain::ports::repositories::feedback::FeedbackRepository;
use spl_domain::ports::repositories::image::ImageRepository;
use spl_domain::ports::repositories::user::UserRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct DbPredictionRepository {
    db: DatabaseConnection,
    user_repository: Arc<dyn UserRepository>,
    image_repository: Arc<dyn ImageRepository>,
    label_repository: Arc<dyn LabelRepository>,
    mark_repository: Arc<dyn PredictionMarkRepository>,
    feedback_repository: Arc<dyn FeedbackRepository>,
}

impl DbPredictionRepository {
    pub fn new(
        db: DatabaseConnection,
        user_repository: Arc<dyn UserRepository>,
        image_repository: Arc<dyn ImageRepository>,
        label_repository: Arc<dyn LabelRepository>,
        mark_repository: Arc<dyn PredictionMarkRepository>,
        feedback_repository: Arc<dyn FeedbackRepository>,
    ) -> Self {
        Self {
            db,
            user_repository,
            image_repository,
            label_repository,
            mark_repository,
            feedback_repository,
        }
    }

    async fn find_relations(
        &self,
        user_id: Uuid,
        image_id: Uuid,
        label_id: i32,
    ) -> Result<(User, Image, Label)> {
        let user_future = self.user_repository.get_by_id(user_id);
        let image_future = self.image_repository.get_by_id(image_id);
        let label_future = self.label_repository.get_by_id(label_id);
        let (user_opt, image_opt, label_opt) =
            tokio::try_join!(user_future, image_future, label_future)?;

        let user =
            user_opt.ok_or_else(|| AppError::NotFound(format!("No user with id {}", user_id)))?;

        let image = image_opt
            .ok_or_else(|| AppError::NotFound(format!("No image with id {}", image_id)))?;

        let label = label_opt
            .ok_or_else(|| AppError::NotFound(format!("No label with id {}", label_id)))?;

        Ok((user, image, label))
    }

    async fn find_marks(&self, prediction_id: Uuid) -> Result<Vec<PredictionMark>> {
        self.mark_repository
            .get_by_prediction_id(prediction_id)
            .await
    }

    async fn find_feedback(&self, prediction_id: Uuid) -> Result<Option<Feedback>> {
        self.feedback_repository
            .get_by_prediction_id(prediction_id)
            .await
    }

    async fn find_marks_map(
        &self,
        prediction_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Vec<PredictionMark>>> {
        if prediction_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let marks = self
            .mark_repository
            .get_by_predictions_ids(prediction_ids)
            .await?
            .into_iter();

        let mut marks_map: HashMap<Uuid, Vec<PredictionMark>> = HashMap::new();

        for mark in marks {
            marks_map.entry(mark.prediction_id).or_default().push(mark);
        }

        Ok(marks_map)
    }

    async fn validate_relations(&self, prediction: &Prediction) -> Result<(User, Image, Label)> {
        self.find_relations(prediction.user.id, prediction.image.id, prediction.label.id)
            .await
    }

    async fn find(&self, select: Select<prediction::Entity>) -> Result<Vec<Prediction>> {
        let models = select
            .find_also_related(label::Entity)
            .find_also_related(image_persistence::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        let prediction_ids: Vec<Uuid> = models.iter().map(|m| m.0.id).collect();
        let user_ids: Vec<Uuid> = models.iter().map(|m| m.0.user_id).collect();

        // 1. Concurrent Fetch Users and Marks
        let (users, marks_map, feedbacks) = tokio::try_join!(
            self.user_repository.get_by_ids(user_ids.clone()),
            self.find_marks_map(prediction_ids.clone()),
            self.feedback_repository
                .get_by_predictions_ids(prediction_ids)
        )?;

        let users_map: HashMap<Uuid, User> = users.into_iter().map(|e| (e.id, e)).collect();
        let feedbacks_map: HashMap<Uuid, Feedback> = feedbacks
            .into_iter()
            .map(|f| (f.prediction_id, f))
            .collect();

        models
            .into_iter()
            .map(|(model, lb_model, im_model)| {
                let user = users_map.get(&model.user_id).cloned().ok_or_else(|| {
                    AppError::NotFound(format!("User with id {} not found", model.user_id))
                });

                let marks = marks_map.get(&model.id).cloned().ok_or_else(|| {
                    AppError::NotFound(format!("Marks not found for prediction id {}", model.id))
                });

                if user.is_ok() && marks.is_ok() {
                    let label = lb_model.ok_or_else(|| {
                        AppError::NotFound(format!(
                            "Label not found for prediction id {}",
                            model.id
                        ))
                    })?;

                    let image = im_model.ok_or_else(|| {
                        AppError::NotFound(format!(
                            "Image not found for prediction id {}",
                            model.id
                        ))
                    })?;

                    let feedback = feedbacks_map.get(&model.id).cloned();

                    let context = PredictionMapperContext {
                        user: user.unwrap(),
                        image: image.into(),
                        label: label.into(),
                        marks: marks.unwrap(),
                        feedback,
                    };

                    model.into_with_context(context)
                } else {
                    Err(AppError::NotFound(format!(
                        "Failed to map prediction with id {} due to missing relations",
                        model.id
                    )))
                }
            })
            .collect()
    }

    async fn with_map<F>(&self, action: F) -> Result<Prediction>
    where
        F: AsyncFnOnce() -> Result<(prediction::Model, User, Image, Label)>,
    {
        let (model, user, image, label) = action().await?;

        let (marks, feedback) =
            tokio::try_join!(self.find_marks(model.id), self.find_feedback(model.id))?;

        let context = PredictionMapperContext {
            user,
            image,
            label,
            marks,
            feedback,
        };

        model.into_with_context(context)
    }
}

#[async_trait::async_trait]
impl CrudRepository<Prediction, Uuid> for DbPredictionRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Prediction>> {
        let result = self
            .find(prediction::Entity::find_by_id(id))
            .await?
            .first()
            .cloned();

        Ok(result)
    }

    async fn create(&self, entity: Prediction) -> Result<Prediction> {
        self.with_map(|| async {
            let (user, image, label) = self.validate_relations(&entity).await?;
            let result =
                crud::create_model::<prediction::Entity, Prediction>(&self.db, entity).await?;
            Ok((result, user, image, label))
        })
        .await
    }

    async fn update(&self, entity: Prediction) -> Result<Prediction> {
        self.with_map(|| async {
            let (user, image, label) = self.validate_relations(&entity).await?;
            let result =
                crud::update_model::<prediction::Entity, Prediction>(&self.db, entity).await?;
            Ok((result, user, image, label))
        })
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<Prediction> {
        let result = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Prediction with id {} not found", id)));

        crud::delete_model::<prediction::Entity, Prediction, Uuid>(&self.db, id).await?;

        result
    }
}

#[async_trait::async_trait]
impl PredictionRepository for DbPredictionRepository {
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Prediction>> {
        self.find(prediction::Entity::find().filter(prediction::Column::UserId.eq(user_id)))
            .await
    }

    async fn get_by_user_id_and_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Prediction>> {
        let result = self
            .find(
                prediction::Entity::find()
                    .filter(prediction::Column::Id.eq(id))
                    .filter(prediction::Column::UserId.eq(user_id)),
            )
            .await?
            .first()
            .cloned();

        Ok(result)
    }

    async fn get_all(&self) -> Result<Vec<Prediction>> {
        self.find(prediction::Entity::find().order_by_desc(prediction::Column::CreatedAt))
            .await
    }

    async fn assign_plot_by_ids_and_user_id(
        &self,
        prediction_ids: Vec<Uuid>,
        user_id: Uuid,
        plot_id: Option<Uuid>,
    ) -> Result<Vec<Prediction>> {
        if prediction_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Update predictions that belong to the user and match the IDs
        prediction::Entity::update_many()
            .col_expr(prediction::Column::PlotId, Expr::value(plot_id))
            .filter(prediction::Column::Id.is_in(prediction_ids.clone()))
            .filter(prediction::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(AppError::from)?;

        // Fetch and return the updated predictions
        self.find(
            prediction::Entity::find()
                .filter(prediction::Column::Id.is_in(prediction_ids))
                .filter(prediction::Column::UserId.eq(user_id)),
        )
        .await
    }

    async fn has_unassigned_predictions(&self, user_id: Uuid) -> Result<bool> {
        let count = prediction::Entity::find()
            .filter(prediction::Column::UserId.eq(user_id))
            .filter(prediction::Column::PlotId.is_null())
            .count(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(count > 0)
    }

    async fn filter(
        &self,
        user_ids: Vec<Uuid>,
        labels: Option<Vec<String>>,
        plot_ids: Option<Vec<Option<Uuid>>>,
        min_date: Option<chrono::DateTime<chrono::Utc>>,
        max_date: Option<chrono::DateTime<chrono::Utc>>,
        offset: u64,
        limit: u64,
    ) -> Result<(u64, Vec<Prediction>)> {
        let query = Self::build_filter_query(user_ids, labels, plot_ids, min_date, max_date);

        // Count total before pagination
        let total = query
            .clone()
            .count(&self.db)
            .await
            .map_err(AppError::from)?;

        let predictions = self
            .find(
                query
                    .order_by_desc(prediction::Column::CreatedAt)
                    .offset(offset)
                    .limit(limit),
            )
            .await?;

        Ok((total, predictions))
    }
}

impl DbPredictionRepository {
    pub fn build_plots_condition(plot_ids: Vec<Option<Uuid>>) -> Condition {
        let mut condition = Condition::any();
        let mut has_null = false;
        let mut ids = Vec::new();

        for pid in plot_ids {
            match pid {
                Some(id) => {
                    if !id.is_nil() {
                        ids.push(id)
                    } else {
                        has_null = true
                    };
                }
                None => has_null = true,
            }
        }

        if !ids.is_empty() {
            condition = condition.add(prediction::Column::PlotId.is_in(ids));
        }
        if has_null {
            condition = condition.add(prediction::Column::PlotId.is_null());
        }

        condition
    }

    pub fn add_filter_query<E>(
        select: Select<E>,
        user_ids: Vec<Uuid>,
        labels: Option<Vec<String>>,
        plot_ids: Option<Vec<Option<Uuid>>>,
        min_date: Option<chrono::DateTime<chrono::Utc>>,
        max_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Select<E>
    where
        E: EntityTrait,
    {
        let mut query = select.filter(prediction::Column::UserId.is_in(user_ids));

        if let Some(labels) = labels {
            query = query.filter(label::Column::Name.is_in(labels));
        }

        if let Some(plot_ids) = plot_ids {
            let condition = DbPredictionRepository::build_plots_condition(plot_ids);
            query = query.filter(condition);
        }

        if let Some(min_date) = min_date {
            query = query.filter(prediction::Column::CreatedAt.gte(min_date));
        }

        if let Some(max_date) = max_date {
            query = query.filter(prediction::Column::CreatedAt.lte(max_date));
        }

        query
    }

    pub fn build_filter_query(
        user_ids: Vec<Uuid>,
        labels: Option<Vec<String>>,
        plot_ids: Option<Vec<Option<Uuid>>>,
        min_date: Option<chrono::DateTime<chrono::Utc>>,
        max_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Select<prediction::Entity> {
        let mut query = prediction::Entity::find();

        if labels.is_some() {
            query = query.join(JoinType::InnerJoin, prediction::Relation::Label.def())
        }

        Self::add_filter_query(
            query, user_ids, labels, plot_ids, // plot_ids will be handled separately
            min_date, max_date,
        )
    }
}
