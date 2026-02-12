use crate::dtos::diagnostics::{CreatePredictionDto, UpdatePredictionDto};
use chrono::Utc;
use spl_domain::entities::diagnostics::{Label, Prediction};
use spl_domain::entities::image::Image;
use spl_domain::entities::user::User;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;

/// Context for creating a Prediction from DTO
pub struct CreatePredictionContext {
    pub user: User,
    pub image: Image,
    pub label: Label,
}

impl IntoWithContext<Prediction, CreatePredictionContext> for CreatePredictionDto {
    type Error = AppError;

    fn into_with_context(self, context: CreatePredictionContext) -> Result<Prediction> {
        Ok(Prediction {
            id: Uuid::new_v4(),
            user: context.user,
            image: context.image,
            label: context.label,
            plot_id: self.plot_id,
            presence_confidence: self.presence_confidence,
            absence_confidence: self.absence_confidence,
            severity: self.severity,
            feedback: None,
            created_at: Utc::now(),
            marks: vec![],
        })
    }
}

/// Context for updating a Prediction
pub struct UpdatePredictionContext {
    pub current: Prediction,
    pub label: Option<Label>,
}

impl IntoWithContext<Prediction, UpdatePredictionContext> for UpdatePredictionDto {
    type Error = AppError;

    fn into_with_context(self, context: UpdatePredictionContext) -> Result<Prediction> {
        Ok(Prediction {
            label: context.label.unwrap_or(context.current.label),
            plot_id: self.plot_id.or(context.current.plot_id),
            ..context.current
        })
    }
}
