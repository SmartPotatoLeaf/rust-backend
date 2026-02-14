use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spl_shared::validation::validate_alphanumeric;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateMarkTypeRequest {
    /// Mark type name (4-32 alphanumeric characters)
    #[validate(length(min = 4, max = 32), custom(function = "validate_alphanumeric"))]
    pub name: String,
    /// Mark type description (max 500 characters)
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateMarkTypeRequest {
    /// New mark type name (4-32 alphanumeric characters)
    #[validate(length(min = 4, max = 32), custom(function = "validate_alphanumeric"))]
    pub name: Option<String>,
    /// New mark type description (max 500 characters)
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct MarkTypeResponse {
    /// Unique identifier of the mark type
    pub id: i32,
    /// Mark type name
    pub name: String,
    /// Mark type description
    pub description: Option<String>,
    /// Timestamp when the mark type was created
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedMarkTypeResponse {
    /// Unique identifier of the mark type
    pub id: i32,
    /// Mark type name
    pub name: String,
}
