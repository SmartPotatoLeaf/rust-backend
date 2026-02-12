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
