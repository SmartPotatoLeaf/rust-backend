use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::category::Category;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recommendation {
    pub id: Uuid,
    pub description: Option<String>,
    pub min_severity: f32,
    pub max_severity: f32,
    pub category: Category,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
