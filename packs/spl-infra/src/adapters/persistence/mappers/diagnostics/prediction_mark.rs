use crate::adapters::persistence::entities::diagnostics::prediction_mark::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::diagnostics::{MarkType, PredictionMark};
use spl_shared::error::AppError;
use spl_shared::traits::IntoWithContext;

pub struct PredictionMarkMapperContext {
    pub mark_type: MarkType,
}

impl IntoWithContext<PredictionMark, PredictionMarkMapperContext> for Model {
    type Error = AppError;

    fn into_with_context(
        self,
        context: PredictionMarkMapperContext,
    ) -> Result<PredictionMark, Self::Error> {
        Ok(PredictionMark {
            id: self.id,
            data: self.data,
            mark_type: context.mark_type,
            prediction_id: self.prediction_id,
            created_at: self.created_at.into(),
        })
    }
}

impl From<PredictionMark> for ActiveModel {
    fn from(entity: PredictionMark) -> Self {
        Self {
            id: Set(entity.id),
            data: Set(entity.data),
            mark_type_id: Set(entity.mark_type.id),
            prediction_id: Set(entity.prediction_id),
            created_at: Set(entity.created_at.into()),
        }
    }
}

