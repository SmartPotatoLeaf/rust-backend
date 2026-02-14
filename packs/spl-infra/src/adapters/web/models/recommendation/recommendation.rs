use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spl_shared::validation::validate_range_min_max;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use crate::adapters::web::models::recommendation::SimplifiedRecommendationCategoryResponse;
use super::category::RecommendationCategoryResponse;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_recommendation"))]
pub struct CreateRecommendationRequest {
    /// Recommendation description (max 1000 characters)
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    /// Category ID for this recommendation
    pub category_id: i32,
    /// Minimum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub min_severity: f32,
    /// Maximum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub max_severity: f32,
}

fn validate_recommendation(req: &CreateRecommendationRequest) -> Result<(), ValidationError> {
    validate_range_min_max(req.min_severity, req.max_severity)
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_update_recommendation"))]
pub struct UpdateRecommendationRequest {
    /// New recommendation description (max 1000 characters)
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    /// New category ID
    pub category_id: Option<i32>,
    /// New minimum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub min_severity: Option<f32>,
    /// New maximum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub max_severity: Option<f32>,
}

fn validate_update_recommendation(
    req: &UpdateRecommendationRequest,
) -> Result<(), ValidationError> {
    if let (Some(min), Some(max)) = (req.min_severity, req.max_severity) {
        validate_range_min_max(min, max)?;
    }
    Ok(())
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct RecommendationResponse {
    /// Unique identifier of the recommendation
    pub id: Uuid,
    /// Recommendation description
    pub description: Option<String>,
    /// Recommendation category
    pub category: RecommendationCategoryResponse,
    /// Minimum severity threshold
    pub min_severity: f32,
    /// Maximum severity threshold
    pub max_severity: f32,
    /// Timestamp when the recommendation was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the recommendation was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedRecommendationResponse {
    /// Unique identifier of the recommendation
    pub id: Uuid,
    /// Recommendation description
    pub description: Option<String>,
    /// Recommendation category (simplified)
    pub category: SimplifiedRecommendationCategoryResponse
}
