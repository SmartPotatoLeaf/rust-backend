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
    /// Optional comment about the prediction (1-500 characters)
    #[validate(length(min = 1, max = 500))]
    pub comment: Option<String>,
    /// Correct label ID according to the user
    #[validate(range(min = 1))]
    pub correct_label_id: i32,
    /// Prediction ID to provide feedback for
    pub prediction_id: Uuid,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateFeedbackRequest {
    /// Updated comment (1-500 characters)
    #[validate(length(min = 1, max = 500))]
    pub comment: Option<String>,
    /// Updated correct label ID
    #[validate(range(min = 1))]
    pub correct_label_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct FeedbackResponse {
    /// Unique identifier of the feedback
    pub id: Uuid,
    /// User comment about the prediction
    pub comment: Option<String>,
    /// Current status of the feedback
    pub status: FeedbackStatusResponse,
    /// Correct label according to the user
    pub correct_label: Option<LabelResponse>,
    /// Associated prediction ID
    pub prediction_id: Uuid,
    /// Timestamp when the feedback was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the feedback was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedFeedbackResponse {
    /// Unique identifier of the feedback
    pub id: Uuid,
    /// User comment about the prediction
    pub comment: Option<String>,
    /// Associated prediction ID
    pub prediction_id: Uuid,
    /// Correct label according to the user (simplified)
    pub correct_label: Option<SimplifiedLabelResponse>,
    /// Current status of the feedback (simplified)
    pub status: SimplifiedFeedbackStatusResponse,
    /// Timestamp when the feedback was last updated
    pub updated_at: DateTime<Utc>,
}
