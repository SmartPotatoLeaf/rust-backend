use serde::{Deserialize, Serialize};
use spl_shared::validation::validate_alphanumeric;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateMarkTypeRequest {
    #[validate(length(min = 4, max = 32), custom(function = "validate_alphanumeric"))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateMarkTypeRequest {
    #[validate(length(min = 4, max = 32), custom(function = "validate_alphanumeric"))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct MarkTypeResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedMarkTypeResponse {
    pub id: i32,
    pub name: String,
}
