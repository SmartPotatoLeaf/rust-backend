use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

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
