use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub filepath: String,
    pub prediction_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
