use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateFeedbackStatusRequest {
    /// Status name (3-16 characters)
    #[validate(length(min = 3, max = 16))]
    pub name: String,
    /// Status description (1-500 characters)
    #[validate(length(min = 1, max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateFeedbackStatusRequest {
    /// New status name (3-16 characters)
    #[validate(length(min = 3, max = 16))]
    pub name: Option<String>,
    /// New status description (1-500 characters)
    #[validate(length(min = 1, max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct FeedbackStatusResponse {
    /// Unique identifier of the feedback status
    pub id: i32,
    /// Status name
    pub name: String,
    /// Status description
    pub description: Option<String>,
    /// Timestamp when the status was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the status was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedFeedbackStatusResponse {
    /// Unique identifier of the feedback status
    pub id: i32,
    /// Status name
    pub name: String,
}

