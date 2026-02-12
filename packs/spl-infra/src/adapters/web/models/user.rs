use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::company::CompanyResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub company_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FullUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub company: Option<CompanyResponse>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,
    pub role: Option<String>,
    pub company_id: Option<Uuid>,
}

