use super::Role;
use crate::entities::company::Company;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub company: Option<Company>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
