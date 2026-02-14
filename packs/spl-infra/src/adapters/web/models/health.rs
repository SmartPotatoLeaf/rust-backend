use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Health check status
    pub status: String,
    /// Health check message
    pub message: String,
}
