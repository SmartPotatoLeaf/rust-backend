use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecommendationDto {
    pub description: Option<String>,
    pub category_id: i32,
    pub min_severity: f32,
    pub max_severity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecommendationDto {
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub min_severity: Option<f32>,
    pub max_severity: Option<f32>,
}
