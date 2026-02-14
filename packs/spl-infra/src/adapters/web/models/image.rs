use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageResponse {
    /// Unique identifier of the image
    pub id: Uuid,
    /// User who uploaded the image
    pub user_id: Uuid,
    /// Original filename of the image
    pub filename: String,
    /// Storage path of the image
    pub filepath: String,
    /// Associated prediction ID
    pub prediction_id: Option<Uuid>,
    /// Timestamp when the image was uploaded
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RawImageResponse {
    /// Base64 encoded image data
    pub data: String,
    /// Original filename
    pub filename: Option<String>,
}

