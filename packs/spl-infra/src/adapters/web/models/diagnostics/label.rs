use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spl_shared::validation::{validate_alphanumeric, validate_range_min_max};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_create_label"))]
pub struct CreateLabelRequest {
    /// Label name (3-64 alphanumeric characters)
    #[validate(length(min = 3, max = 64), custom(function = "validate_alphanumeric"))]
    pub name: String,
    /// Label description (max 500 characters)
    #[validate(length(max = 500))]
    pub description: Option<String>,
    /// Minimum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub min: f32,
    /// Maximum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub max: f32,
    /// Label weight for classification priority
    #[validate(range(min = 0))]
    pub weight: i32,
}

fn validate_create_label(req: &CreateLabelRequest) -> Result<(), ValidationError> {
    validate_range_min_max(req.min, req.max)
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_update_label"))]
pub struct UpdateLabelRequest {
    /// New label name (3-64 alphanumeric characters)
    #[validate(length(min = 3, max = 64), custom(function = "validate_alphanumeric"))]
    pub name: Option<String>,
    /// New label description (max 500 characters)
    #[validate(length(max = 500))]
    pub description: Option<String>,
    /// New minimum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub min: Option<f32>,
    /// New maximum severity threshold (0.0-100.0)
    #[validate(range(min = 0.0, max = 100.0))]
    pub max: Option<f32>,
    /// New label weight
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
    /// Unique identifier of the label
    pub id: i32,
    /// Label name
    pub name: String,
    /// Label description
    pub description: Option<String>,
    /// Minimum severity threshold
    pub min: f32,
    /// Maximum severity threshold
    pub max: f32,
    /// Label weight for classification
    pub weight: i32,
    /// Timestamp when the label was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the label was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Deserialize, Clone)]
pub struct SimplifiedLabelResponse {
    /// Unique identifier of the label
    pub id: i32,
    /// Label name
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema, Deserialize, Clone)]
pub struct RawLabelResponse {
    /// Label name
    pub name: String,
    /// Label description
    pub description: Option<String>,
    /// Minimum severity threshold
    pub min: f32,
    /// Maximum severity threshold
    pub max: f32,
    /// Label weight for classification
    pub weight: i32,
}