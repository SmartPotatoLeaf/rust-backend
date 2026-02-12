use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateImageDto {
    pub filename: String,
    pub filepath: String,
    pub user_id: Uuid,
    pub prediction_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ImageDto {
    pub id: Uuid,
    pub filename: String,
    pub filepath: String,
    pub user_id: Uuid,
    pub prediction_id: Option<Uuid>,
}
