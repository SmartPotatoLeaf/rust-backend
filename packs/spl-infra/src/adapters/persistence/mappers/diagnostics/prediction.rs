use crate::adapters::persistence::entities::diagnostics::prediction::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::diagnostics::{Label, Prediction, PredictionMark};
use spl_domain::entities::image::Image;
use spl_domain::entities::user::User;
use spl_shared::error::AppError;
use spl_shared::traits::IntoWithContext;

/// Context for mapping a Prediction from database model
pub struct PredictionMapperContext {
    pub user: User,
    pub image: Image,
    pub label: Label,
    pub marks: Vec<PredictionMark>,
}

impl IntoWithContext<Prediction, PredictionMapperContext> for Model {
    type Error = AppError;

    fn into_with_context(
        self,
        context: PredictionMapperContext,
    ) -> Result<Prediction, Self::Error> {
        Ok(Prediction {
            id: self.id,
            user: context.user,
            image: context.image,
            label: context.label,
            marks: context.marks,
            plot_id: self.plot_id,
            presence_confidence: self.presence_confidence,
            absence_confidence: self.absence_confidence,
            severity: self.severity,
            created_at: self.created_at.into(),
        })
    }
}

impl From<Prediction> for ActiveModel {
    fn from(entity: Prediction) -> Self {
        Self {
            id: Set(entity.id),
            user_id: Set(entity.user.id),
            image_id: Set(entity.image.id),
            label_id: Set(entity.label.id),
            plot_id: Set(entity.plot_id),
            presence_confidence: Set(entity.presence_confidence),
            absence_confidence: Set(entity.absence_confidence),
            severity: Set(entity.severity),
            created_at: Set(entity.created_at.into()),
        }
    }
}

impl From<Prediction> for Model {
    fn from(entity: Prediction) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user.id,
            image_id: entity.image.id,
            label_id: entity.label.id,
            plot_id: entity.plot_id,
            presence_confidence: entity.presence_confidence,
            absence_confidence: entity.absence_confidence,
            severity: entity.severity,
            created_at: entity.created_at.into(),
        }
    }
}
