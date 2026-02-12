use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateFeedbackStatusRequest {
    #[validate(length(min = 3, max = 16))]
    pub name: String,
    #[validate(length(min = 1, max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateFeedbackStatusRequest {
    #[validate(length(min = 3, max = 16))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct FeedbackStatusResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct SimplifiedFeedbackStatusResponse {
    pub id: i32,
    pub name: String,
}

