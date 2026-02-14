use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spl_shared::maps_to;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub min: f32,
    pub max: f32,
    pub weight: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simplified version of Label for public predictions (without DB metadata)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawLabel {
    pub name: String,
    pub description: Option<String>,
    pub min: f32,
    pub max: f32,
}


maps_to!(
    RawLabel {
        name, description, min, max
    } #from [ Label ]
);