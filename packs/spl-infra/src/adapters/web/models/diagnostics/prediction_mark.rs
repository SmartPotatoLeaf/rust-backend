use crate::adapters::web::models::diagnostics::SimplifiedMarkTypeResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PredictionMarkResponse {
    /// Unique identifier of the prediction mark
    pub id: Uuid,
    /// Mark data (filepath, filename, etc.)
    pub data: serde_json::Value,
    /// Type of segmentation mark
    pub mark_type: SimplifiedMarkTypeResponse,
    /// Associated prediction ID
    pub prediction_id: Uuid,
    /// Timestamp when the mark was created
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RawPredictionMarkResponse {
    /// Base64 encoded mask image data
    pub data: String,
    /// Mark type name (e.g., "leaf", "lesion")
    pub mark_type: String,
}
