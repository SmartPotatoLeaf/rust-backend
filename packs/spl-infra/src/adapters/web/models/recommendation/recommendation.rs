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
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub category_id: i32,
    #[validate(range(min = 0.0, max = 100.0))]
    pub min_severity: f32,
    #[validate(range(min = 0.0, max = 100.0))]
    pub max_severity: f32,
}

fn validate_recommendation(req: &CreateRecommendationRequest) -> Result<(), ValidationError> {
    validate_range_min_max(req.min_severity, req.max_severity)
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_update_recommendation"))]
pub struct UpdateRecommendationRequest {
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub category_id: Option<i32>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub min_severity: Option<f32>,
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
    pub id: Uuid,
    pub description: Option<String>,
    pub category: RecommendationCategoryResponse,
    pub min_severity: f32,
    pub max_severity: f32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedRecommendationResponse {
    pub id: Uuid,
    pub description: Option<String>,
    pub category: SimplifiedRecommendationCategoryResponse
}
