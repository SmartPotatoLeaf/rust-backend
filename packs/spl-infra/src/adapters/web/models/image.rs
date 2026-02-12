use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageResponse {
    pub id: Uuid,
    pub filename: String,
    pub filepath: String,
    pub created_at: DateTime<Utc>,
}
