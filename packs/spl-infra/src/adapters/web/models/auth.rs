use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub company_id: Option<Uuid>,
    pub role: Option<String>,
}
