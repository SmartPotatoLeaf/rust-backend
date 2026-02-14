use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::{Uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub filepath: String,
    pub prediction_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Simplified version of Image for public predictions (with base64 data instead of file paths)
#[derive(Debug, Clone)]
pub struct RawImage {
    /// Image data in bytes
    pub data: Bytes,
    /// Original filename (if provided)
    pub filename: Option<String>,
}

