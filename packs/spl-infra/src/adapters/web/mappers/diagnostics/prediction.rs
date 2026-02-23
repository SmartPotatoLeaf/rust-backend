use crate::adapters::web::models::diagnostics::{
    FilterPredictionsRequest, PredictionDetailedResponse, PredictionResponse,
    RawPredictionResponse, SimplifiedPredictionDetailedResponse, SimplifiedPredictionResponse,
};
use spl_application::dtos::diagnostics::FilterPredictionDto;
use spl_domain::entities::diagnostics::prediction::{PredictionDetailed, RawPrediction};
use spl_domain::entities::diagnostics::Prediction;
use spl_domain::entities::user::User;
use spl_shared::error::AppError;
use spl_shared::traits::IntoWithContext;

impl From<Prediction> for PredictionResponse {
    fn from(param: Prediction) -> Self {
        Self {
            id: param.id,
            user: param.user.into(),
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
            user: param.user.into(),
            presence_confidence: param.presence_confidence,
            absence_confidence: param.absence_confidence,
            plot_id: param.plot_id,
            severity: param.severity,
            label: param.label.into(),
            image: param.image.into(),
            marks: param.marks.into_iter().map(Into::into).collect(),
            created_at: param.created_at,
            feedback: param.feedback.map(Into::into),
        }
    }
}

impl From<RawPrediction> for RawPredictionResponse {
    fn from(value: RawPrediction) -> Self {
        Self {
            presence_confidence: value.presence_confidence,
            absence_confidence: value.absence_confidence,
            severity: value.severity,
            created_at: value.created_at,
            image: value.image.into(),
            label: value.label.into(),
            marks: value.marks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PredictionDetailed> for PredictionDetailedResponse {
    fn from(param: PredictionDetailed) -> Self {
        Self {
            id: param.id,
            user: param.user.into(),
            presence_confidence: param.presence_confidence,
            absence_confidence: param.absence_confidence,
            plot_id: param.plot_id,
            severity: param.severity,
            label: param.label.into(),
            image: param.image.into(),
            marks: param.marks.into_iter().map(Into::into).collect(),
            created_at: param.created_at,
            feedback: param.feedback.map(Into::into),
            recommendations: param.recommendations.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PredictionDetailed> for SimplifiedPredictionDetailedResponse {
    fn from(param: PredictionDetailed) -> Self {
        Self {
            id: param.id,
            user: param.user.into(),
            presence_confidence: param.presence_confidence,
            absence_confidence: param.absence_confidence,
            plot_id: param.plot_id,
            severity: param.severity,
            label: param.label.into(),
            image: param.image.into(),
            marks: param.marks.into_iter().map(Into::into).collect(),
            created_at: param.created_at,
            feedback: param.feedback.map(Into::into),
            recommendations: param.recommendations.into_iter().map(Into::into).collect(),
        }
    }
}
