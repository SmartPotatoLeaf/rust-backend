use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePredictionDto {
    pub user_id: Uuid,
    pub image_id: Uuid,
    pub label_id: i32,
    pub plot_id: Option<Uuid>,
    pub presence_confidence: f32,
    pub absence_confidence: f32,
    pub severity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePredictionDto {
    pub label_id: Option<i32>,
    pub plot_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct FilterPredictionDto {
    pub requester_id: Uuid,
    pub company_id: Option<Uuid>,
    pub target_user_ids: Option<Vec<Uuid>>,
    pub labels: Option<Vec<String>>,
    pub plot_ids: Option<Vec<Option<Uuid>>>,
    pub min_date: Option<chrono::DateTime<chrono::Utc>>,
    pub max_date: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<u64>,
    pub page: Option<u64>,
}
