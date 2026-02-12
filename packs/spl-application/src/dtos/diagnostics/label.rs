use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLabelDto {
    pub name: String,
    pub description: Option<String>,
    pub min: f32,
    pub max: f32,
    pub weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLabelDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub weight: Option<i32>,
}
