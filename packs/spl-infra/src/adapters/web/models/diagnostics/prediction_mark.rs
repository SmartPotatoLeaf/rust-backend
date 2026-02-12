use crate::adapters::web::models::diagnostics::SimplifiedMarkTypeResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PredictionMarkResponse {
    pub id: Uuid,
    pub data: serde_json::Value,
    pub mark_type: SimplifiedMarkTypeResponse,
    pub created_at: DateTime<Utc>,
}
