use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::status::FeedbackStatus;
use crate::entities::diagnostics::Label;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feedback {
    pub id: Uuid,
    pub comment: Option<String>,
    pub status: FeedbackStatus,
    pub correct_label: Option<Label>,
    pub prediction_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
