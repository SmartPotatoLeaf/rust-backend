use chrono::{DateTime, Utc};
use crate::adapters::web::models::diagnostics::{LabelResponse, SimplifiedLabelResponse};
use crate::adapters::web::models::feedback::status::{
    FeedbackStatusResponse, SimplifiedFeedbackStatusResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateFeedbackRequest {
    #[validate(length(min = 1, max = 500))]
    pub comment: Option<String>,
    #[validate(range(min = 1))]
    pub correct_label_id: i32,
    pub prediction_id: Uuid,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateFeedbackRequest {
    #[validate(length(min = 1, max = 500))]
    pub comment: Option<String>,
    #[validate(range(min = 1))]
    pub correct_label_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct FeedbackResponse {
    pub id: Uuid,
    pub comment: Option<String>,
    pub status: FeedbackStatusResponse,
    pub correct_label: Option<LabelResponse>,
    pub prediction_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedFeedbackResponse {
    pub id: Uuid,
    pub comment: Option<String>,
    pub prediction_id: Uuid,
    pub correct_label: Option<SimplifiedLabelResponse>,
    pub status: SimplifiedFeedbackStatusResponse,
    pub updated_at: DateTime<Utc>,
}
