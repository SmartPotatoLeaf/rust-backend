use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateFeedbackDto {
    pub comment: Option<String>,
    pub correct_label_id: i32,
    pub prediction_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateFeedbackDto {
    pub comment: Option<String>,
    pub correct_label_id: Option<i32>,
}
