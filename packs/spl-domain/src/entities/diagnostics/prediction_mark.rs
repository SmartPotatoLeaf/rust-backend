use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::MarkType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionMark {
    pub id: Uuid,
    pub data: serde_json::Value,
    pub mark_type: MarkType,
    pub prediction_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Simplified version of PredictionMark for public predictions (with base64 mask data)
#[derive(Debug, Clone,)]
pub struct RawPredictionMark {
    /// Mask image bytes
    pub data: Bytes,
    /// Type of mask (e.g., "leaf_mask", "lt_blg_lesion_mask")
    pub mark_type: String,
}

