use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::company::CompanyResponse;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    /// Unique identifier of the user
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: Option<String>,
    /// User's first name
    pub name: Option<String>,
    /// User's last name
    pub surname: Option<String>,
    /// User's role name
    pub role: String,
    /// Company ID the user belongs to
    pub company_id: Option<Uuid>,
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimplifiedUserResponse {
    /// Unique identifier of the user
    pub id: Uuid,
    /// Username
    pub username: String,
    /// User's first name
    pub name: Option<String>,
    /// User's last name
    pub surname: Option<String>,
    /// User's role name
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FullUserResponse {
    /// Unique identifier of the user
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: Option<String>,
    /// User's first name
    pub name: Option<String>,
    /// User's last name
    pub surname: Option<String>,
    /// User's role name
    pub role: String,
    /// Full company information
    pub company: Option<CompanyResponse>,
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    /// New username (3-32 characters)
    #[validate(length(min = 3, max = 32))]
    pub username: Option<String>,
    /// New email address
    #[validate(email)]
    pub email: Option<String>,
    /// New password (8-128 characters)
    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,
    /// New first name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// New last name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    pub surname: Option<String>,
    /// New role name
    pub role: Option<String>,
    /// New company ID
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    /// New email address
    #[validate(email)]
    pub email: Option<String>,
    /// New first name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// New last name (1-100 characters)
    #[validate(length(min = 1, max = 100))]
    pub surname: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    /// Current password for verification (8-128 characters)
    #[validate(length(min = 8, max = 128))]
    pub current_password: String,
    /// New password (8-128 characters)
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct RoleResponse {
    /// Unique identifier of the role
    pub id: i32,
    /// Role name
    pub name: String,
    /// Role level
    pub level: i16,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedRoleResponse {
    /// Unique identifier of the role
    pub id: i32,
    /// Role name
    pub name: String,
}
