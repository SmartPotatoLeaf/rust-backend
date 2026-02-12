use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateFeedbackStatusDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateFeedbackStatusDto {
    pub name: Option<String>,
    pub description: Option<String>,
}
