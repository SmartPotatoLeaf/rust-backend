use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCompanyRequest {
    /// Company name (3-100 characters)
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    /// Optional company description (max 500 characters)
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateCompanyRequest {
    /// New company name (3-100 characters)
    #[validate(length(min = 3, max = 100))]
    pub name: Option<String>,
    /// New company description (max 500 characters)
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct CompanyResponse {
    /// Unique identifier of the company
    pub id: Uuid,
    /// Company name
    pub name: String,
    /// Company description
    pub description: Option<String>,
    /// Timestamp when the company was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the company was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedCompanyResponse {
    /// Unique identifier of the company
    pub id: Uuid,
    /// Company name
    pub name: String,
}
