use crate::dtos::feedback::feedback::{CreateFeedbackDto, UpdateFeedbackDto};
use chrono::Utc;
use spl_domain::entities::diagnostics::Label;
use spl_domain::entities::feedback::{Feedback, FeedbackStatus};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::FromWithContext;
use uuid::Uuid;

// CreateFeedbackDto needs context (status, label)
pub struct CreateFeedbackContext {
    pub status: FeedbackStatus,
    pub label: Label,
}

impl FromWithContext<CreateFeedbackDto, CreateFeedbackContext> for Feedback {
    type Error = AppError;

    fn from_with_context(dto: CreateFeedbackDto, ctx: CreateFeedbackContext) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            comment: dto.comment,
            status: ctx.status,
            correct_label: Some(ctx.label),
            prediction_id: dto.prediction_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

// UpdateFeedbackDto needs context (current feedback, optional new label)
pub struct UpdateFeedbackContext {
    pub current: Feedback,
    pub label: Option<Label>,
}

impl FromWithContext<UpdateFeedbackDto, UpdateFeedbackContext> for Feedback {
    type Error = AppError;

    fn from_with_context(dto: UpdateFeedbackDto, ctx: UpdateFeedbackContext) -> Result<Self> {
        Ok(Feedback {
            comment: dto.comment.or(ctx.current.comment),
            correct_label: ctx.label.or(ctx.current.correct_label),
            updated_at: Utc::now(),
            ..ctx.current
        })
    }
}
