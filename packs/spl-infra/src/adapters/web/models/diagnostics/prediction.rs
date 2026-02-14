use crate::adapters::web::models::diagnostics::{
    prediction_mark::{PredictionMarkResponse, RawPredictionMarkResponse},
    LabelResponse, SimplifiedLabelResponse,
};
use crate::adapters::web::models::feedback::{FeedbackResponse, SimplifiedFeedbackResponse};
use crate::adapters::web::models::image::{ImageResponse, RawImageResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spl_shared::validation::validate_range_min_max;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use crate::adapters::web::models::diagnostics::label::RawLabelResponse;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PredictionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub presence_confidence: f32,
    pub absence_confidence: f32,
    pub severity: f32,
    pub created_at: DateTime<Utc>,
    pub plot_id: Option<Uuid>,
    pub image: ImageResponse,
    pub label: LabelResponse,
    pub marks: Vec<PredictionMarkResponse>,
    pub feedback: Option<FeedbackResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimplifiedPredictionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub presence_confidence: f32,
    pub absence_confidence: f32,
    pub severity: f32,
    pub label: SimplifiedLabelResponse,
    pub plot_id: Option<Uuid>,
    pub image: ImageResponse,
    pub marks: Vec<PredictionMarkResponse>,
    pub feedback: Option<SimplifiedFeedbackResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_filter_predictions"))]
pub struct FilterPredictionsRequest {
    pub company_id: Option<Uuid>,
    pub user_ids: Option<Vec<Uuid>>,
    pub labels: Option<Vec<String>>,
    pub plot_ids: Option<Vec<Option<Uuid>>>,
    pub min_date: Option<DateTime<Utc>>,
    pub max_date: Option<DateTime<Utc>>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u64>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
}

fn validate_filter_predictions(req: &FilterPredictionsRequest) -> Result<(), ValidationError> {
    if let (Some(min), Some(max)) = (req.min_date, req.max_date) {
        validate_range_min_max(min, max)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PredictionsListResponse {
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub items: Vec<PredictionResponse>,
}

#[derive(ToSchema)]
pub struct CreatePredictionRequest {
    #[schema(format = "binary")]
    pub file: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RawPredictionResponse {
    pub presence_confidence: f32,
    pub absence_confidence: f32,
    pub severity: f32,
    pub created_at: DateTime<Utc>,
    pub image: RawImageResponse,
    pub label: RawLabelResponse,
    pub marks: Vec<RawPredictionMarkResponse>,
}
