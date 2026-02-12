use crate::dtos::feedback::feedback::{CreateFeedbackDto, UpdateFeedbackDto};
use crate::mappers::feedback::feedback::{CreateFeedbackContext, UpdateFeedbackContext};
use spl_domain::entities::feedback::Feedback;
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::diagnostics::LabelRepository;
use spl_domain::ports::repositories::feedback::{FeedbackRepository, FeedbackStatusRepository};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

pub struct FeedbackService {
    feedback_repo: Arc<dyn FeedbackRepository>,
    status_repo: Arc<dyn FeedbackStatusRepository>,
    label_repo: Arc<dyn LabelRepository>,
}

impl FeedbackService {
    pub fn new(
        feedback_repo: Arc<dyn FeedbackRepository>,
        status_repo: Arc<dyn FeedbackStatusRepository>,
        label_repo: Arc<dyn LabelRepository>,
    ) -> Self {
        Self {
            feedback_repo,
            status_repo,
            label_repo,
        }
    }

    pub async fn create_by_user(&self, user: &User, dto: CreateFeedbackDto) -> Result<Feedback> {
        // Check if feedback already exists
        let existing = self
            .feedback_repo
            .get_by_user_and_prediction_id(user.id, dto.prediction_id)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(format!(
                "Feedback already exists for prediction {}",
                dto.prediction_id
            )));
        }

        let label_future = self.label_repo.get_by_id(dto.correct_label_id);
        let status_future = self.status_repo.get_by_name("pending"); // Default status name

        let (label_opt, status_opt) = tokio::try_join!(label_future, status_future)?;

        // Validate label
        let label = label_opt.ok_or_else(|| {
            AppError::ValidationError(format!(
                "Label with id {} does not exist",
                dto.correct_label_id
            ))
        })?;

        // Get default status
        let status = status_opt
            .ok_or_else(|| AppError::NotFound("Default feedback status not found".to_string()))?;

        // Create feedback with context
        let context = CreateFeedbackContext { status, label };

        let feedback = dto.into_with_context(context)?;

        self.feedback_repo.create(feedback).await
    }

    pub async fn get_by_id_and_user(&self, id: Uuid, user: &User) -> Result<Option<Feedback>> {
        self.feedback_repo.get_by_id_and_user_id(id, user.id).await
    }

    pub async fn get_all_by_user(&self, user: &User) -> Result<Vec<Feedback>> {
        self.feedback_repo.get_all_by_user_id(user.id).await
    }

    pub async fn get_by_user_and_prediction(
        &self,
        user: &User,
        prediction_id: Uuid,
    ) -> Result<Option<Feedback>> {
        self.feedback_repo
            .get_by_user_and_prediction_id(user.id, prediction_id)
            .await
    }

    pub async fn update_by_user(
        &self,
        id: Uuid,
        user: &User,
        dto: UpdateFeedbackDto,
    ) -> Result<Feedback> {
        let current_future = self.feedback_repo.get_by_id_and_user_id(id, user.id);

        let label_future = if let Some(label_id) = dto.correct_label_id {
            Some(self.label_repo.get_by_id(label_id))
        } else {
            None
        };

        let current_opt;
        let mut label_opt = None;

        if let Some(label_fut) = label_future {
            (current_opt, label_opt) = tokio::try_join!(current_future, label_fut)?;
        } else {
            current_opt = current_future.await?;
        }

        let current = current_opt.ok_or_else(|| {
            AppError::NotFound(format!(
                "Feedback with id {} not found for user {}",
                id, user.id
            ))
        })?;

        // Get label if provided
        let label = if dto.correct_label_id.is_some() {
            Some(label_opt.ok_or_else(|| {
                AppError::ValidationError(format!("Label with id {:?} does not exist", dto.correct_label_id))
            })?)
        } else {
            None
        };

        let context = UpdateFeedbackContext { current, label };
        let updated = dto.into_with_context(context)?;

        self.feedback_repo.update(updated).await
    }

    pub async fn delete_by_user(&self, id: Uuid, user: &User) -> Result<Feedback> {
        self.feedback_repo.delete_by_id_and_user_id(id, user.id).await
    }
}
