use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRecommendationCategoryRequest {
    /// Category name (3-64 characters)
    #[validate(length(min = 3, max = 64))]
    pub name: String,
    /// Category description
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRecommendationCategoryRequest {
    /// New category name (3-64 characters)
    #[validate(length(min = 3, max = 64))]
    pub name: Option<String>,
    /// New category description
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct RecommendationCategoryResponse {
    /// Unique identifier of the recommendation category
    pub id: i32,
    /// Category name
    pub name: String,
    /// Category description
    pub description: Option<String>,
    /// Timestamp when the category was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the category was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedRecommendationCategoryResponse {
    /// Unique identifier of the recommendation category
    pub id: i32,
    /// Category name
    pub name: String,
}
