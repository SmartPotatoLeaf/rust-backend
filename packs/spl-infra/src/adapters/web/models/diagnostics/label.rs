use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spl_shared::validation::{validate_alphanumeric, validate_range_min_max};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_create_label"))]
pub struct CreateLabelRequest {
    #[validate(length(min = 3, max = 64), custom(function = "validate_alphanumeric"))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub min: f32,
    #[validate(range(min = 0.0, max = 100.0))]
    pub max: f32,
    #[validate(range(min = 0))]
    pub weight: i32,
}

fn validate_create_label(req: &CreateLabelRequest) -> Result<(), ValidationError> {
    validate_range_min_max(req.min, req.max)
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_update_label"))]
pub struct UpdateLabelRequest {
    #[validate(length(min = 3, max = 64), custom(function = "validate_alphanumeric"))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub min: Option<f32>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub max: Option<f32>,
    #[validate(range(min = 0))]
    pub weight: Option<i32>,
}

fn validate_update_label(req: &UpdateLabelRequest) -> Result<(), ValidationError> {
    if let (Some(min), Some(max)) = (req.min, req.max) {
        validate_range_min_max(min, max)?;
    }
    Ok(())
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct LabelResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub min: f32,
    pub max: f32,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Deserialize, Clone)]
pub struct SimplifiedLabelResponse {
    pub id: i32,
    pub name: String,
}
