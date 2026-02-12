use sea_orm::Set;
use spl_domain::entities::diagnostics::Label;
use spl_domain::entities::feedback::{Feedback, FeedbackStatus};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::FromWithContext;

use crate::adapters::persistence::entities::feedback::feedback::{ActiveModel, Model};

pub struct FeedbackMapperContext {
    pub status: FeedbackStatus,
    pub correct_label: Option<Label>,
}

impl FromWithContext<Model, FeedbackMapperContext> for Feedback {
    type Error = AppError;

    fn from_with_context(model: Model, context: FeedbackMapperContext) -> Result<Feedback> {
        Ok(Feedback {
            id: model.id,
            comment: model.comment,
            status: context.status,
            correct_label: context.correct_label,
            prediction_id: model.prediction_id,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        })
    }
}

impl From<Feedback> for ActiveModel {
    fn from(entity: Feedback) -> Self {
        Self {
            id: Set(entity.id),
            comment: Set(entity.comment),
            status_id: Set(entity.status.id),
            correct_label_id: Set(entity.correct_label.map(|l| l.id)),
            prediction_id: Set(entity.prediction_id),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}
