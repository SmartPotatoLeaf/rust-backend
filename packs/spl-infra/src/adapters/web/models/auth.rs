use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub company_id: Option<Uuid>,
}

impl LoginRequest {
    /// Validates that at least one of username or email is provided
    pub fn validate_identifier(&self) -> Result<(), String> {
        if self.username.is_none() && self.email.is_none() {
            return Err("Either username or email must be provided".to_string());
        }
        Ok(())
    }
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
