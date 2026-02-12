use crate::adapters::web::models::diagnostics::{
    FilterPredictionsRequest, PredictionResponse, SimplifiedPredictionResponse,
};
use spl_application::dtos::diagnostics::FilterPredictionDto;
use spl_domain::entities::diagnostics::Prediction;
use spl_domain::entities::user::User;
use spl_shared::error::AppError;
use spl_shared::traits::IntoWithContext;

impl From<Prediction> for PredictionResponse {
    fn from(param: Prediction) -> Self {
        Self {
            id: param.id,
            user_id: param.user.id,
            presence_confidence: param.presence_confidence,
            absence_confidence: param.absence_confidence,
            severity: param.severity,
            created_at: param.created_at,
            plot_id: param.plot_id,
            image: param.image.into(),
            label: param.label.into(),
            marks: param.marks.into_iter().map(Into::into).collect(),
            feedback: param.feedback.map(Into::into),
        }
    }
}

pub struct FilterPredictionMapperContext {
    pub requester: User,
}

impl IntoWithContext<FilterPredictionDto, FilterPredictionMapperContext>
    for FilterPredictionsRequest
{
    type Error = AppError;

    fn into_with_context(
        self,
        context: FilterPredictionMapperContext,
    ) -> Result<FilterPredictionDto, Self::Error> {
        Ok(FilterPredictionDto {
            requester_id: context.requester.id,
            company_id: self.company_id,
            target_user_ids: self.user_ids,
            labels: self.labels,
            plot_ids: self.plot_ids,
            min_date: self.min_date,
            max_date: self.max_date,
            limit: self.limit,
            page: self.page,
        })
    }
}

impl From<Prediction> for SimplifiedPredictionResponse {
    fn from(param: Prediction) -> Self {
        Self {
            id: param.id,
            user_id: param.user.id,
            presence_confidence: param.presence_confidence,
            absence_confidence: param.absence_confidence,
            severity: param.severity,
            label: param.label.into(),
            image: param.image.into(),
            marks: param.marks.into_iter().map(Into::into).collect(),
            created_at: param.created_at,
            feedback: param.feedback.map(Into::into),
        }
    }
}
