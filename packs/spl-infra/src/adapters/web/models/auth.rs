use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Username for authentication (3-32 characters)
    #[validate(length(min = 3, max = 32))]
    pub username: Option<String>,
    /// Email address for authentication
    #[validate(email)]
    pub email: Option<String>,
    /// User password (8-128 characters)
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    /// Optional company ID for multi-tenant login
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
    /// JWT authentication token
    pub token: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// Unique username (3-32 characters)
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    /// Email address
    #[validate(email)]
    pub email: Option<String>,
    /// User password (8-128 characters)
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    /// User's first name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// User's last name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    pub surname: Option<String>,
    /// Company ID to associate with the user
    pub company_id: Option<Uuid>,
    /// Role name to assign to the user
    pub role: Option<String>,
}
